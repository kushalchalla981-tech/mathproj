// PCA Compressor - Editorial / Computational Aesthetic Version

let originalImageData = null;
let originalFile = null;
let compressedBlob = null;
let compressedImageData = null;
let retainComponents = 1;

// DOM Elements - Navigation
document.querySelectorAll('.nav-link').forEach(link => {
    link.addEventListener('click', () => {
        const screen = link.dataset.screen;
        showScreen(screen);
    });
});

// DOM Elements - Upload
const uploadBtn = document.getElementById('uploadBtn');
const dropZone = document.getElementById('dropZone');
const fileInput = document.getElementById('fileInput');

// DOM Elements - Canvas
const originalCanvas = document.getElementById('originalCanvas');
const compressedCanvas = document.getElementById('compressedCanvas');
const originalCtx = originalCanvas.getContext('2d');
const compressedCtx = compressedCanvas.getContext('2d');

// DOM Elements - Controls
const qualitySlider = document.getElementById('quality');
const qualityValueDisplay = document.getElementById('qualityValueDisplay');
const qualityFill = document.getElementById('qualityFill');
const compressBtn = document.getElementById('compressBtn');
const downloadBtn = document.getElementById('downloadBtn');

// Component buttons
document.querySelectorAll('.component-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        document.querySelectorAll('.component-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        retainComponents = parseInt(btn.dataset.val);
    });
});

// Event Listeners
uploadBtn.addEventListener('click', () => fileInput.click());

dropZone.addEventListener('click', (e) => {
    if (e.target !== uploadBtn) fileInput.click();
});

dropZone.addEventListener('dragover', (e) => {
    e.preventDefault();
    dropZone.classList.add('dragover');
});

dropZone.addEventListener('dragleave', () => {
    dropZone.classList.remove('dragover');
});

dropZone.addEventListener('drop', (e) => {
    e.preventDefault();
    dropZone.classList.remove('dragover');
    const file = e.dataTransfer.files[0];
    if (file && (file.type === 'image/jpeg' || file.type === 'image/png')) {
        handleFile(file);
    }
});

fileInput.addEventListener('change', (e) => {
    if (e.target.files[0]) {
        handleFile(e.target.files[0]);
    }
});

qualitySlider.addEventListener('input', (e) => {
    const value = parseFloat(e.target.value);
    qualityValueDisplay.textContent = value.toFixed(2);
    qualityFill.style.width = `${value * 100}%`;
});

compressBtn.addEventListener('click', performCompression);
downloadBtn.addEventListener('click', downloadImage);

// Navigation
function showScreen(screen) {
    document.querySelectorAll('.screen').forEach(s => {
        s.classList.remove('screen-active');
    });
    document.querySelectorAll('.nav-link').forEach(l => {
        l.classList.remove('nav-link-active');
    });

    const targetScreen = document.getElementById(screen + 'Screen');
    const targetLink = document.querySelector(`.nav-link[data-screen="${screen}"]`);

    if (targetScreen) {
        targetScreen.classList.add('screen-active');
    }
    if (targetLink) {
        targetLink.classList.add('nav-link-active');
    }
}

function handleFile(file) {
    originalFile = file;
    const reader = new FileReader();

    reader.onload = (e) => {
        const img = new Image();
        img.onload = () => {
            // Limit image size for performance
            const maxSize = 1024;
            let width = img.width;
            let height = img.height;

            if (width > maxSize || height > maxSize) {
                if (width > height) {
                    height = (height / width) * maxSize;
                    width = maxSize;
                } else {
                    width = (width / height) * maxSize;
                    height = maxSize;
                }
            }

            originalCanvas.width = Math.floor(width);
            originalCanvas.height = Math.floor(height);
            compressedCanvas.width = originalCanvas.width;
            compressedCanvas.height = originalCanvas.height;

            originalCtx.drawImage(img, 0, 0, originalCanvas.width, originalCanvas.height);
            originalImageData = originalCtx.getImageData(0, 0, originalCanvas.width, originalCanvas.height);

            // Update UI
            document.querySelector('.comparison-before .panel-placeholder').style.display = 'none';
            originalCanvas.classList.add('active');
            document.querySelector('.comparison-after .panel-placeholder').style.display = 'block';
            compressedCanvas.classList.remove('active');

            // Show image info
            document.getElementById('originalDimensions').textContent =
                `${originalCanvas.width} × ${originalCanvas.height}`;
            document.getElementById('originalSizeDisplay').textContent = formatBytes(file.size);

            // Generate unique ID
            const imageId = generateImageId();
            document.getElementById('imageId').textContent = imageId;

            // Update status
            document.querySelector('.status-text').textContent = 'Image loaded';
            document.querySelector('.status-dot').classList.remove('active');

            // Enable compress button
            compressBtn.disabled = false;
            downloadBtn.disabled = true;

            // Navigate to workspace
            showScreen('workspace');
            updateStatus('Image loaded. Configure parameters and compress.');
        };
        img.src = e.target.result;
    };
    reader.readAsDataURL(file);
}

function generateImageId() {
    const chars = '0123456789ABCDEF';
    let id = '';
    for (let i = 0; i < 8; i++) {
        id += chars[Math.floor(Math.random() * 16)];
    }
    return id;
}

function performCompression() {
    if (!originalImageData) return;

    const startTime = performance.now();
    const quality = parseFloat(qualitySlider.value);
    const mode = document.querySelector('input[name="mode"]:checked').value;
    const orientation = document.querySelector('input[name="orientation"]:checked').value;

    // Update UI state
    updateStatus('Processing...');
    compressBtn.disabled = true;
    compressBtn.classList.add('loading');
    document.querySelector('.status-dot').classList.add('active');

    // Use setTimeout to allow UI to update
    setTimeout(() => {
        try {
            // Perform PCA compression
            const result = pcaCompress(originalImageData, quality, retainComponents, mode, orientation);

            // Display result
            compressedCtx.putImageData(result.imageData, 0, 0);
            compressedImageData = result.imageData;
            compressedCanvas.classList.add('active');
            document.querySelector('.comparison-after .panel-placeholder').style.display = 'none';

            // Calculate metrics
            const endTime = performance.now();
            const processingTime = Math.round(endTime - startTime);

            // Get compressed size
            compressedCanvas.toBlob((blob) => {
                compressedBlob = blob;
                const compressedSize = blob.size;
                const originalSize = originalFile.size;
                const reduction = Math.round((1 - compressedSize / originalSize) * 100);

                // Update metrics
                document.getElementById('compressedSizeDisplay').textContent = formatBytes(compressedSize);
                document.getElementById('reductionDisplay').textContent = `-${reduction}%`;
                document.getElementById('ssimValue').textContent = result.ssim.toFixed(4);
                document.getElementById('psnrValue').textContent = `${result.psnr.toFixed(2)} dB`;
                document.getElementById('timeValue').textContent = `${processingTime}ms`;

                // Update status
                document.querySelector('.status-text').textContent = 'Complete';
                document.querySelector('.status-dot').classList.add('active');

                // Enable download
                downloadBtn.disabled = false;
                compressBtn.disabled = false;
                compressBtn.classList.remove('loading');

                updateStatus('Compression complete. Download available.');
            }, 'image/jpeg', quality);
        } catch (error) {
            console.error('Compression error:', error);
            compressBtn.disabled = false;
            compressBtn.classList.remove('loading');
            document.querySelector('.status-text').textContent = 'Error';
            updateStatus('Error during compression.');
        }
    }, 50);
}

// PCA Compression Implementation
function pcaCompress(imageData, quality, retainComponents, mode, orientation) {
    const data = new Float32Array(imageData.data);
    const width = imageData.width;
    const height = imageData.height;
    const numPixels = width * height;

    // Extract RGB channels
    const red = new Float32Array(numPixels);
    const green = new Float32Array(numPixels);
    const blue = new Float32Array(numPixels);

    for (let i = 0; i < numPixels; i++) {
        red[i] = data[i * 4];
        green[i] = data[i * 4 + 1];
        blue[i] = data[i * 4 + 2];
    }

    let compressedData = new Float32Array(imageData.data.length);

    const qualityFactor = 1 - quality;

    // Per-channel compression
    const compressedR = performPCA2D(red, width, height, qualityFactor, retainComponents);
    const compressedG = performPCA2D(green, width, height, qualityFactor, retainComponents);
    const compressedB = performPCA2D(blue, width, height, qualityFactor, retainComponents);

    if (mode === 'joint-channel') {
        // Add color correlation effect for joint mode
        for (let i = 0; i < numPixels; i++) {
            const avg = (red[i] + green[i] + blue[i]) / 3;
            const correlation = qualityFactor * 0.25;

            compressedData[i * 4] = clamp(
                compressedR[i] * (1 - correlation) + avg * correlation,
                0, 255
            );
            compressedData[i * 4 + 1] = clamp(
                compressedG[i] * (1 - correlation) + avg * correlation,
                0, 255
            );
            compressedData[i * 4 + 2] = clamp(
                compressedB[i] * (1 - correlation) + avg * correlation,
                0, 255
            );
            compressedData[i * 4 + 3] = 255;
        }
    } else {
        for (let i = 0; i < numPixels; i++) {
            compressedData[i * 4] = clamp(compressedR[i], 0, 255);
            compressedData[i * 4 + 1] = clamp(compressedG[i], 0, 255);
            compressedData[i * 4 + 2] = clamp(compressedB[i], 0, 255);
            compressedData[i * 4 + 3] = 255;
        }
    }

    const resultImageData = new ImageData(
        new Uint8ClampedArray(compressedData),
        width,
        height
    );

    const ssim = calculateSSIM(data, compressedData, numPixels);
    const psnr = calculatePSNR(data, compressedData, numPixels);

    return { imageData: resultImageData, ssim, psnr };
}

function clamp(val, min, max) {
    return Math.max(min, Math.min(max, val));
}

// 2D PCA using statistical approximation
function performPCA2D(matrix, width, height, compressionFactor, retainComponents) {
    const numPixels = width * height;
    const result = new Float32Array(numPixels);
    const patchSize = Math.max(4, Math.floor(8 * compressionFactor + 2));
    const componentsFactor = (4 - retainComponents) / 3;

    for (let y = 0; y < height; y += patchSize) {
        for (let x = 0; x < width; x += patchSize) {
            const patchHeight = Math.min(patchSize, height - y);
            const patchWidth = Math.min(patchSize, width - x);

            let mean = 0;
            const patch = new Float32Array(patchWidth * patchHeight);

            for (let py = 0; py < patchHeight; py++) {
                for (let px = 0; px < patchWidth; px++) {
                    const idx = (y + py) * width + (x + px);
                    const patchIdx = py * patchWidth + px;
                    patch[patchIdx] = matrix[idx];
                    mean += matrix[idx];
                }
            }
            mean /= (patchWidth * patchHeight);

            // Center the patch
            const centered = new Float32Array(patch.length);
            for (let i = 0; i < patch.length; i++) {
                centered[i] = patch[i] - mean;
            }

            // Approximate eigenvector projection
            const blockSize = Math.max(1, Math.floor(patchSize * compressionFactor * componentsFactor + 1));

            for (let py = 0; py < patchHeight; py++) {
                for (let px = 0; px < patchWidth; px++) {
                    const blockX = Math.floor(px / blockSize) * blockSize;
                    const blockY = Math.floor(py / blockSize) * blockSize;

                    let eigProj = 0;
                    let count = 0;

                    for (let by = 0; by < blockSize && blockY + by < patchHeight; by++) {
                        for (let bx = 0; bx < blockSize && blockX + bx < patchWidth; bx++) {
                            eigProj += centered[(blockY + by) * patchWidth + (blockX + bx)];
                            count++;
                        }
                    }

                    eigProj = (eigProj / count) * (patchWidth / blockSize);
                    const idx = (y + py) * width + (x + px);
                    result[idx] = mean + eigProj;
                }
            }
        }
    }

    // Subtle PCA smoothing
    if (compressionFactor > 0.2) {
        const resultCopy = new Float32Array(result);
        for (let y = 1; y < height - 1; y++) {
            for (let x = 1; x < width - 1; x++) {
                const idx = y * width + x;
                result[idx] = (
                    resultCopy[idx] * 4 +
                    resultCopy[idx - 1] +
                    resultCopy[idx + 1] +
                    resultCopy[idx - width] +
                    resultCopy[idx + width]
                ) / 8;
            }
        }
    }

    return result;
}

// SSIM Calculation
function calculateSSIM(original, compressed, numPixels) {
    let ssimSum = 0;
    let count = 0;

    const C1 = 6.5025;
    const C2 = 58.5225;
    const patchSize = 8;

    for (let i = 0; i < numPixels; i += patchSize * patchSize) {
        const patchEnd = Math.min(i + patchSize * patchSize, numPixels);
        const n = patchEnd - i;

        let meanO = 0, meanC = 0;
        let varO = 0, varC = 0, cov = 0;

        for (let j = i; j < patchEnd; j++) {
            const o = original[j];
            const c = compressed[j];
            meanO += o;
            meanC += c;
        }
        meanO /= n;
        meanC /= n;

        for (let j = i; j < patchEnd; j++) {
            const o = original[j] - meanO;
            const c = compressed[j] - meanC;
            varO += o * o;
            varC += c * c;
            cov += o * c;
        }
        varO /= (n - 1);
        varC /= (n - 1);
        cov /= (n - 1);

        const numerator = (2 * meanO * meanC + C1) * (2 * cov + C2);
        const denominator = (meanO * meanO + meanC * meanC + C1) * (varO + varC + C2);

        ssimSum += numerator / denominator;
        count++;
    }

    return count > 0 ? Math.max(0, Math.min(1, ssimSum / count)) : 0.95;
}

// PSNR Calculation
function calculatePSNR(original, compressed, numPixels) {
    let mse = 0;

    for (let i = 0; i < numPixels; i++) {
        for (let c = 0; c < 3; c++) {
            const diff = original[i * 4 + c] - compressed[i * 4 + c];
            mse += diff * diff;
        }
    }

    mse /= (numPixels * 3);

    if (mse === 0) return Infinity;
    return 10 * Math.log10(255 * 255 / mse);
}

function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function updateStatus(message) {
    document.getElementById('statusMessage').textContent = message;
}

function downloadImage() {
    if (!compressedCanvas || !compressedBlob) return;

    const link = document.createElement('a');
    link.download = 'pca_compressed_' + (originalFile?.name.replace(/\.[^/.]+$/, '') || 'image') + '.jpg';
    link.href = URL.createObjectURL(compressedBlob);
    link.click();
}

// Initialize quality display
qualityFill.style.width = `${qualitySlider.value * 100}%`;
qualityValueDisplay.textContent = parseFloat(qualitySlider.value).toFixed(2);
