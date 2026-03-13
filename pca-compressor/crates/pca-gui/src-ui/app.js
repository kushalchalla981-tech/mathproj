// App state
const state = {
    currentScreen: 'home',
    selectedFile: null,
    originalImageData: null,
    compressedImageData: null,
    compressionParams: {
        quality: 0.7,
        mode: 'per-channel',
        retain_components: 1,
        orientation: 'auto',
        tile_size: null,
        max_memory_mb: 1024,
        output_format: 'jpeg',
        strip_metadata: false
    },
    settings: {
        maxMemory: 1024,
        threadCount: 4,
        autoRecompress: true,
        defaultFormat: 'jpeg'
    },
    recentFiles: []
};

// Initialize Tauri API
let tauriAPI = null;
let dialogAPI = null;
let fsAPI = null;

// Try to detect if running in Tauri
function isTauriApp() {
    return window.__TAURI__ !== undefined;
}

// Initialize app when Tauri is ready
if (window.__TAURI__) {
    tauriAPI = window.__TAURI__.core;
    dialogAPI = window.__TAURI__.dialog;
    fsAPI = window.__TAURI__.fs;
}

// Screen navigation
function showScreen(screenName) {
    // Hide all screens
    document.querySelectorAll('.screen').forEach(screen => {
        screen.classList.remove('active');
    });

    // Show target screen
    document.getElementById(`${screenName}Screen`).classList.add('active');

    // Update nav buttons
    document.querySelectorAll('.nav-buttons button').forEach(btn => {
        btn.classList.remove('active');
    });
    document.getElementById(`${screenName}Btn`).classList.add('active');

    state.currentScreen = screenName;
}

// File handling
async function selectFile() {
    if (!dialogAPI) return;

    try {
        const selected = await dialogAPI.open({
            multiple: false,
            filters: [{
                name: 'Images',
                extensions: ['jpg', 'jpeg', 'png']
            }]
        });

        if (selected && selected.length > 0) {
            await loadFile(selected[0]);
        }
    } catch (error) {
        console.error('Error selecting file:', error);
        showError('Failed to select file');
    }
}

async function loadFile(filePath) {
    try {
        state.selectedFile = filePath;

        // Read file as data URL for preview
        const fileData = await fsAPI.readBinaryFile(filePath);
        const blob = new Blob([fileData], { type: 'image/jpeg' });
        const dataUrl = URL.createObjectURL(blob);

        state.originalImageData = dataUrl;
        document.getElementById('originalPreview').innerHTML = `<img src="${dataUrl}" alt="Original image">`;

        // Get image info
        if (tauriAPI) {
            const info = await tauriAPI.invoke('get_image_info', { path: filePath });
            document.getElementById('imageInfo').textContent =
                `${info.width}x${info.height} • ${info.size_human}`;
        }

        // Enable compress button
        document.getElementById('compressBtn').disabled = false;

        // Switch to workspace
        showScreen('workspace');

        // Add to recent files
        addToRecentFiles(filePath, info);
    } catch (error) {
        console.error('Error loading file:', error);
        showError('Failed to load image');
    }
}

// Compression
async function compressImage() {
    if (!state.selectedFile) return;

    const outputExtension = state.compressionParams.output_format === 'png' ? 'png' : 'jpg';
    const outputPath = state.selectedFile.replace(/\.[^.]+$/, `_compressed.${outputExtension}`);

    try {
        setStatus('Compressing...');
        document.getElementById('compressBtn').disabled = true;

        const params = { ...state.compressionParams };

        const result = await tauriAPI.invoke('compress_image', {
            inputPath: state.selectedFile,
            outputPath: outputPath,
            params: params
        });

        if (result.success) {
            // Update UI with results
            updateMetrics(result);

            // Load compressed image preview
            const compressedData = await fsAPI.readBinaryFile(outputPath);
            const blob = new Blob([compressedData], { type: 'image/jpeg' });
            const dataUrl = URL.createObjectURL(blob);
            state.compressedImageData = dataUrl;

            document.getElementById('compressedPreview').innerHTML = `<img src="${dataUrl}" alt="Compressed image">`;

            // Enable export button
            document.getElementById('exportBtn').disabled = false;

            setStatus('Compression complete');
        } else {
            showError(result.error || 'Compression failed');
        }
    } catch (error) {
        console.error('Compression error:', error);
        showError('Compression failed');
    } finally {
        document.getElementById('compressBtn').disabled = false;
    }
}

function updateMetrics(metrics) {
    document.getElementById('originalSize').textContent = formatBytes(metrics.original_size);
    document.getElementById('compressedSize').textContent = formatBytes(metrics.compressed_size);
    document.getElementById('compressionRatio').textContent = `${metrics.compression_ratio.toFixed(2)}x`;
    document.getElementById('ssim').textContent = metrics.ssim.toFixed(3);
    document.getElementById('psnr').textContent = `${metrics.psnr.toFixed(1)} dB`;
    document.getElementById('processingTime').textContent = `${metrics.processing_time_ms} ms`;
}

function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

// Export
async function exportImage() {
    if (!state.compressedImageData) return;

    if (!dialogAPI) {
        // Fallback: download directly
        const link = document.createElement('a');
        link.href = state.compressedImageData;
        link.download = 'compressed.jpg';
        link.click();
        return;
    }

    try {
        const outputExtension = state.compressionParams.output_format === 'png' ? 'png' : 'jpg';
        const outputPath = await dialogAPI.save({
            filters: [{
                name: outputExtension.toUpperCase(),
                extensions: [outputExtension]
            }]
        });

        if (outputPath) {
            const fileData = await fsAPI.readBinaryFile(
                state.selectedFile.replace(/\.[^.]+$/, `_compressed.${outputExtension}`)
            );
            await fsAPI.writeBinaryFile(outputPath, fileData);
            setStatus('Image exported');
        }
    } catch (error) {
        console.error('Export error:', error);
        showError('Failed to export image');
    }
}

// Recent files
function addToRecentFiles(filePath, info) {
    const existing = state.recentFiles.find(f => f.path === filePath);
    if (existing) {
        // Move to front
        state.recentFiles = state.recentFiles.filter(f => f.path !== filePath);
        state.recentFiles.unshift(existing);
    } else {
        state.recentFiles.unshift({
            path: filePath,
            name: filePath.split(/[/\\]/).pop(),
            size: info ? info.size_human : 'Unknown',
            timestamp: Date.now()
        });
    }

    // Keep only last 10
    state.recentFiles = state.recentFiles.slice(0, 10);
    updateRecentFilesDisplay();
}

function updateRecentFilesDisplay() {
    const container = document.getElementById('recentFilesList');
    container.innerHTML = state.recentFiles.map(file => `
        <div class="file-item" data-path="${file.path}">
            <div class="file-info">
                <span class="file-name">${file.name}</span>
                <span class="file-meta">${file.size}</span>
            </div>
            <span class="file-time">${new Date(file.timestamp).toLocaleDateString()}</span>
        </div>
    `).join('');

    // Add click handlers
    container.querySelectorAll('.file-item').forEach(item => {
        item.addEventListener('click', () => {
            const path = item.getAttribute('data-path');
            loadFile(path);
        });
    });
}

// Status messages
function setStatus(message) {
    document.getElementById('statusText').textContent = message;
}

function showError(message) {
    setStatus('Error: ' + message);
    setTimeout(() => setStatus('Ready'), 5000);
}

// Settings
function loadSettings() {
    const saved = localStorage.getItem('pca-compressor-settings');
    if (saved) {
        Object.assign(state.settings, JSON.parse(saved));
    }

    // Update UI
    document.getElementById('maxMemory').value = state.settings.maxMemory;
    document.getElementById('threadCount').value = state.settings.threadCount;
    document.getElementById('autoRecompress').checked = state.settings.autoRecompress;
    document.getElementById('defaultFormat').value = state.settings.defaultFormat;
}

function saveSettings() {
    state.settings.maxMemory = parseInt(document.getElementById('maxMemory').value);
    state.settings.threadCount = parseInt(document.getElementById('threadCount').value);
    state.settings.autoRecompress = document.getElementById('autoRecompress').checked;
    state.settings.defaultFormat = document.getElementById('defaultFormat').value;

    localStorage.setItem('pca-compressor-settings', JSON.stringify(state.settings));

    setStatus('Settings saved');
}

function resetSettings() {
    state.settings = {
        maxMemory: 1024,
        threadCount: 4,
        autoRecompress: true,
        defaultFormat: 'jpeg'
    };

    document.getElementById('maxMemory').value = state.settings.maxMemory;
    document.getElementById('threadCount').value = state.settings.threadCount;
    document.getElementById('autoRecompress').checked = state.settings.autoRecompress;
    document.getElementById('defaultFormat').value = state.settings.defaultFormat;

    saveSettings();
}

// Event listeners
function setupEventListeners() {
    // Navigation
    document.getElementById('homeBtn').addEventListener('click', () => showScreen('home'));
    document.getElementById('workspaceBtn').addEventListener('click', () => showScreen('workspace'));
    document.getElementById('settingsBtn').addEventListener('click', () => showScreen('settings'));

    // File selection
    document.getElementById('selectFileBtn').addEventListener('click', selectFile);
    document.getElementById('fileInput').addEventListener('change', (e) => {
        if (e.target.files.length > 0) {
            loadFile(e.target.files[0]);
        }
    });

    // Drag and drop
    const dropZone = document.getElementById('dropZone');
    dropZone.addEventListener('dragover', (e) => {
        e.preventDefault();
        dropZone.classList.add('drag-over');
    });

    dropZone.addEventListener('dragleave', () => {
        dropZone.classList.remove('drag-over');
    });

    dropZone.addEventListener('drop', async (e) => {
        e.preventDefault();
        dropZone.classList.remove('drag-over');

        if (e.dataTransfer.files.length > 0) {
            if (dialogAPI) {
                // In Tauri, we need to get the actual file path
                // This would need additional Tauri API calls
                showError('Drag and drop not fully supported in Tauri yet');
            } else {
                // Web version
                loadFile(e.dataTransfer.files[0]);
            }
        }
    });

    // Compression controls
    const qualitySlider = document.getElementById('quality');
    qualitySlider.addEventListener('input', (e) => {
        const value = parseFloat(e.target.value);
        document.getElementById('qualityValue').textContent = value.toFixed(1);
        state.compressionParams.quality = value;

        if (state.settings.autoRecompress && state.selectedFile && state.compressedImageData) {
            compressImage();
        }
    });

    document.querySelectorAll('input[name="mode"]').forEach(radio => {
        radio.addEventListener('change', (e) => {
            state.compressionParams.mode = e.target.value;

            if (state.settings.autoRecompress && state.selectedFile && state.compressedImageData) {
                compressImage();
            }
        });
    });

    document.getElementById('retainComponents').addEventListener('change', (e) => {
        state.compressionParams.retain_components = parseInt(e.target.value);

        if (state.settings.autoRecompress && state.selectedFile && state.compressedImageData) {
            compressImage();
        }
    });

    document.getElementById('orientation').addEventListener('change', (e) => {
        state.compressionParams.orientation = e.target.value;

        if (state.settings.autoRecompress && state.selectedFile && state.compressedImageData) {
            compressImage();
        }
    });

    document.getElementById('tileProcessing').addEventListener('change', (e) => {
        const tileSizeInput = document.getElementById('tileSize');
        tileSizeInput.disabled = !e.target.checked;

        if (e.target.checked) {
            const tileSize = parseInt(tileSizeInput.value) || 1024;
            state.compressionParams.tile_size = tileSize;
        } else {
            state.compressionParams.tile_size = null;
        }

        if (state.settings.autoRecompress && state.selectedFile && state.compressedImageData) {
            compressImage();
        }
    });

    document.getElementById('tileSize').addEventListener('change', (e) => {
        if (document.getElementById('tileProcessing').checked) {
            state.compressionParams.tile_size = parseInt(e.target.value) || 1024;

            if (state.settings.autoRecompress && state.selectedFile && state.compressedImageData) {
                compressImage();
            }
        }
    });

    // Action buttons
    document.getElementById('compressBtn').addEventListener('click', compressImage);
    document.getElementById('exportBtn').addEventListener('click', exportImage);

    // Settings
    document.getElementById('saveSettingsBtn').addEventListener('click', saveSettings);
    document.getElementById('resetSettingsBtn').addEventListener('click', resetSettings);
}

// Initialize app
async function init() {
    loadSettings();
    setupEventListeners();

    if (tauriAPI) {
        try {
            const version = await tauriAPI.invoke('get_version');
            console.log(`PCA Compressor v${version}`);
        } catch (error) {
            console.error('Failed to get version:', error);
        }
    }

    console.log('PCA Compressor initialized');
}

// Start app when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}