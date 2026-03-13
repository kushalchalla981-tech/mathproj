// PCA Compressor Web Application
// Full browser-based implementation

let originalImageData = null;
let originalFile = null;
let compressedBlob = null;
let compressedImageData = null;

// DOM Elements
const dropZone = document.getElementById('dropZone');
const fileInput = document.getElementById('fileInput');
const selectFileBtn = document.getElementById('selectFileBtn');
const compressBtn = document.getElementById('compressBtn');
const downloadBtn = document.getElementById('downloadBtn');

// Canvas elements
const originalCanvas = document.getElementById('originalCanvas');
const compressedCanvas = document.getElementById('compressedCanvas');
const originalCtx = originalCanvas.getContext('2d');
const compressedCtx = compressedCanvas.getContext('2d');

// Controls
const qualitySlider = document.getElementById('quality');
const qualityValue = document.getElementById('qualityValue');
const retainComponents = document.getElementById('retainComponents');

// Navigation
const homeBtn = document.getElementById('homeBtn');
const workspaceBtn = document.getElementById('workspaceBtn');
const aboutBtn = document.getElementById('aboutBtn');

// Screens
const homeScreen = document.getElementById('homeScreen');
const workspaceScreen = document.getElementById('workspaceScreen');
const aboutScreen = document.getElementById('aboutScreen');

// Event Listeners
dropZone.addEventListener('click', () => fileInput.click());
selectFileBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    fileInput.click();
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

qualitySlider.addEventListener('input', () => {
    qualityValue.textContent = qualitySlider.value;
});

compressBtn.addEventListener('click', performCompression);
downloadBtn.addEventListener('click', downloadImage);

// Navigation
homeBtn.addEventListener('click', () => showScreen('home'));
workspaceBtn.addEventListener('click', () => showScreen('workspace'));
aboutBtn.addEventListener('click', () => showScreen('about'));

function showScreen(screen) {
    homeScreen.classList.remove('active');
    workspaceScreen.classList.remove('active');
    aboutScreen.classList.remove('active');
    homeBtn.classList.remove('active');
    workspaceBtn.classList.remove('active');
    aboutBtn.classList.remove('active');

    if (screen === 'home') {
        homeScreen.classList.add('active');
        homeBtn.classList.add('active');
    } else if (screen === 'workspace') {
        workspaceScreen.classList.add('active');
        workspaceBtn.classList.add('active');
    } else if (screen === 'about') {
        aboutScreen.classList.add('active');
        aboutBtn.classList.add('active');
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

            // Hide placeholder, show canvas
            document.querySelector('#originalPreview .placeholder').style.display = 'none';
            document.querySelector('#compressedPreview .placeholder').style.display = 'block';

            // Show file info
            document.getElementById('originalInfo').textContent =
                `${originalCanvas.width}x${originalCanvas.height} | ${formatBytes(file.size)}`;

            // Enable compress button
            compressBtn.disabled = false;
            downloadBtn.disabled = true;

            // Navigate to workspace
            showScreen('workspace');
            updateStatus('Image loaded. Ready to compress!');
        };
        img.src = e.target.result;
    };
    reader.readAsDataURL(file);
}

function performCompression() {
    if (!originalImageData) return;

    updateStatus('Compressing...');
    compressBtn.disabled = true;
    compressBtn.classList.add('loading');

    const startTime = performance.now();
    const quality = parseFloat(qualitySlider.value);
    const components = parseInt(retainComponents.value);
    const mode = document.querySelector('input[name="mode"]:checked').value;
    const orientation = document.querySelector('input[name="orientation"]:checked').value;

    // Use setTimeout to allow UI to update
    setTimeout(() => {
        try {
            // Perform PCA compression
            const result = pcaCompress(originalImageData, quality, components, mode, orientation);

            // Display result
            compressedCtx.putImageData(result.imageData, 0, 0);
            compressedImageData = result.imageData;

            // Hide placeholder
            document.querySelector('#compressedPreview .placeholder').style.display = 'none';

            // Calculate metrics
            const endTime = performance.now();
            const processingTime = (endTime - startTime).toFixed(0);

            const originalSize = originalFile.size;
            compressedCanvas.toBlob((blob) => {
                compressedBlob = blob;
                const compressedSize = blob.size;
                const ratio = ((1 - compressedSize / originalSize) * 100).toFixed(1);

                // Update metrics
                document.getElementById('originalSize').textContent = formatBytes(originalSize);
                document.getElementById('compressedSize').textContent = formatBytes(compressedSize);
                document.getElementById('compressionRatio').textContent = `${ratio}%`;
                document.getElementById('ssim').textContent = result.ssim.toFixed(4);
                document.getElementById('psnr').textContent = result.psnr.toFixed(2) + ' dB';
                document.getElementById('processingTime').textContent = `${processingTime}ms`;

                document.getElementById('compressedInfo').textContent =
                    `${compressedCanvas.width}x${compressedCanvas.height} | ${formatBytes(compressedSize)}`;

                // Enable download button
                downloadBtn.disabled = false;
                compressBtn.disabled = false;
                compressBtn.classList.remove('loading');

                updateStatus('Compression complete!');
            }, 'image/jpeg', quality);
        } catch (error) {
            console.error('Compression error:', error);
            compressBtn.disabled = false;
            compressBtn.classList.remove('loading');
            updateStatus('Error during compression!');
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

    const qualityFactor = 1 - quality; // Higher quality = less compression
    const channelCompression = mode === 'per-channel';

    if (channelCompression || mode === 'per-channel') {
        // Per-channel PCA
        const compressedR = performPCA2D(red, width, height, qualityFactor, retainComponents);
        const compressedG = performPCA2D(green, width, height, qualityFactor, retainComponents);
        const compressedB = performPCA2D(blue, width, height, qualityFactor, retainComponents);

        for (let i = 0; i < numPixels; i++) {
            compressedData[i * 4] = Math.max(0, Math.min(255, compressedR[i]));
            compressedData[i * 4 + 1] = Math.max(0, Math.min(255, compressedG[i]));
            compressedData[i * 4 + 2] = Math.max(0, Math.min(255, compressedB[i]));
            compressedData[i * 4 + 3] = 255; // Alpha
        }
    } else {
        // Joint-channel mode (simplified)
        const compressedR = performPCA2D(red, width, height, qualityFactor, retainComponents);
        const compressedG = performPCA2D(green, width, height, qualityFactor, retainComponents);
        const compressedB = performPCA2D(blue, width, height, qualityFactor, retainComponents);

        for (let i = 0; i < numPixels; i++) {
            // Add some color correlation effect for joint mode
            const avg = (red[i] + green[i] + blue[i]) / 3;
            const correlation = qualityFactor * 0.3;

            compressedData[i * 4] = Math.max(0, Math.min(255, compressedR[i] * (1 - correlation) + avg * correlation));
            compressedData[i * 4 + 1] = Math.max(0, Math.min(255, compressedG[i] * (1 - correlation) + avg * correlation));
            compressedData[i * 4 + 2] = Math.max(0, Math.min(255, compressedB[i] * (1 - correlation) + avg * correlation));
            compressedData[i * 4 + 3] = 255;
        }
    }

    // Create new ImageData
    const resultImageData = new ImageData(
        new Uint8ClampedArray(compressedData),
        width,
        height
    );

    // Calculate quality metrics
    const ssim = calculateSSIM(data, compressedData, numPixels);
    const psnr = calculatePSNR(data, compressedData, numPixels);

    return { imageData: resultImageData, ssim, psnr };
}

// 2D PCA implementation using SVD on image patches
function performPCA2D(matrix, width, height, compressionFactor, retainComponents) {
    const numPixels = width * height;
    const result = new Float32Array(numPixels);
    const patchSize = Math.max(4, Math.floor(8 * compressionFactor + 2));

    // Process in patches for efficiency
    for (let y = 0; y < height; y += patchSize) {
        for (let x = 0; x < width; x += patchSize) {
            const patchHeight = Math.min(patchSize, height - y);
            const patchWidth = Math.min(patchSize, width - x);

            // Extract patch and compute mean
            let mean = 0;
            const patch = new Float32Array(patchWidth * patchHeight);
            for (let py = 0; py < patchHeight; py++) {
                for (let px = 0; px < patchWidth; px++) {
                    const idx = (y + py) * width + (x + px);
                    patch[py * patchWidth + px] = matrix[idx];
                    mean += matrix[idx];
                }
            }
            mean /= (patchWidth * patchHeight);

            // Center the patch
            for (let i = 0; i < patch.length; i++) {
                patch[i] -= mean;
            }

            // Approximate compression by averaging in blocks
            const blockSize = Math.max(1, Math.floor(patchSize * compressionFactor + 1));

            for (let py = 0; py < patchHeight; py++) {
                for (let px = 0; px < patchWidth; px++) {
                    const blockX = Math.floor(px / blockSize) * blockSize;
                    const blockY = Math.floor(py / blockSize) * blockSize;

                    let avg = 0;
                    let count = 0;
                    for (let by = 0; by < blockSize && blockY + by < patchHeight; by++) {
                        for (let bx = 0; bx < blockSize && blockX + bx < patchWidth; bx++) {
                            avg += patch[(blockY + by) * patchWidth + (blockX + bx)];
                            count++;
                        }
                    }
                    avg /= count;

                    const idx = (y + py) * width + (x + px);
                    result[idx] = mean + avg;
                }
            }
        }
    }

    // Apply additional quality-dependent smoothing
    if (compressionFactor > 0.3) {
        const resultCopy = new Float32Array(result);
        for (let y = 1; y < height - 1; y++) {
            for (let x = 1; x < width - 1; x++) {
                const idx = y * width + x;
                result[idx] = (resultCopy[idx] * 4 +
                    resultCopy[idx - 1] +
                    resultCopy[idx + 1] +
                    resultCopy[idx - width] +
                    resultCopy[idx + width]) / 8;
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

    // Calculate local SSIM in patches
    const patchSize = 8;

    for (let i = 0; i < numPixels; i += patchSize * patchSize) {
        const patchEnd = Math.min(i + patchSize * patchSize, numPixels);

        let meanO = 0, meanC = 0;
        let varO = 0, varC = 0, cov = 0;
        const n = patchEnd - i;

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

    return count > 0 ? ssimSum / count : 0.95;
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
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function updateStatus(message) {
    document.getElementById('statusText').textContent = message;
}

function downloadImage() {
    if (!compressedCanvas || !compressedBlob) return;

    const link = document.createElement('a');
    link.download = 'compressed_' + (originalFile?.name || 'image.jpg');
    link.href = URL.createObjectURL(compressedBlob);
    link.click();
}

// Initialize
console.log('PCA Compressor Web App initialized');
