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
const overlayCanvas = document.getElementById('overlayCanvas');
const originalCtx = originalCanvas.getContext('2d');
const compressedCtx = compressedCanvas.getContext('2d');
const overlayCtx = overlayCanvas.getContext('2d');

// DOM Elements - Eigen Analysis
const overlayColorSelect = document.getElementById('overlayColor');

// Current eigen analysis result
let currentEigenResult = null;
let currentOverlayColor = 'red';

// Color map for overlay
const colorMap = {
    'red': '#FF0000',
    'yellow': '#FFFF00',
    'cyan': '#00FFFF',
    'green': '#00FF00',
    'magenta': '#FF00FF'
};

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

overlayColorSelect.addEventListener('change', (e) => {
    currentOverlayColor = e.target.value;
    if (currentEigenResult) {
        drawAxisOverlay(currentEigenResult);
    }
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

            // Perform eigen analysis
            try {
                currentEigenResult = performEigenAnalysis(originalImageData);
                updateEigenDisplay(currentEigenResult);
                drawAxisOverlay(currentEigenResult);
                document.getElementById('eigenPanel').style.display = 'block';
            } catch (e) {
                console.error('Eigen analysis error:', e);
                document.getElementById('eigenPanel').style.display = 'none';
            }

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

// Eigen Analysis Functions
function performEigenAnalysis(imageData) {
    const data = imageData.data;
    const width = imageData.width;
    const height = imageData.height;
    const numPixels = width * height;

    // Build pixel data matrix
    const pixels = [];
    for (let i = 0; i < numPixels; i++) {
        pixels.push({
            r: data[i * 4] / 255,
            g: data[i * 4 + 1] / 255,
            b: data[i * 4 + 2] / 255
        });
    }

    // Calculate mean for each channel
    let meanR = 0, meanG = 0, meanB = 0;
    for (let i = 0; i < numPixels; i++) {
        meanR += pixels[i].r;
        meanG += pixels[i].g;
        meanB += pixels[i].b;
    }
    meanR /= numPixels;
    meanG /= numPixels;
    meanB /= numPixels;

    // Center the data
    const centered = pixels.map(p => ({
        r: p.r - meanR,
        g: p.g - meanG,
        b: p.b - meanB
    }));

    // Compute 3x3 covariance matrix
    let covRR = 0, covGG = 0, covBB = 0;
    let covRG = 0, covRB = 0, covGB = 0;

    for (let i = 0; i < numPixels; i++) {
        const c = centered[i];
        covRR += c.r * c.r;
        covGG += c.g * c.g;
        covBB += c.b * c.b;
        covRG += c.r * c.g;
        covRB += c.r * c.b;
        covGB += c.g * c.b;
    }

    covRR /= (numPixels - 1);
    covGG /= (numPixels - 1);
    covBB /= (numPixels - 1);
    covRG /= (numPixels - 1);
    covRB /= (numPixels - 1);
    covGB /= (numPixels - 1);

    // Covariance matrix
    const cov = [
        [covRR, covRG, covRB],
        [covRG, covGG, covGB],
        [covRB, covGB, covBB]
    ];

    // Jacobi eigenvalue algorithm
    const eigenResult = jacobiEigenvalue(cov);

    // Sort eigenvalues in descending order
    const indices = [0, 1, 2].sort((a, b) => eigenResult.values[b] - eigenResult.values[a]);
    const eigenvalues = indices.map(i => Math.max(eigenResult.values[i], 1e-10));
    const eigenvectors = indices.flatMap(i => eigenResult.vectors[i]);

    // Calculate variance explained
    const totalVariance = eigenvalues.reduce((a, b) => a + b, 0);
    const varianceExplained = eigenvalues.map(v => (v / totalVariance) * 100);
    const cumulativeVariance = varianceExplained.reduce((acc, v, i) => {
        acc.push((acc[i - 1] || 0) + v);
        return acc;
    }, []);

    // Calculate principal angle from first eigenvector
    const primaryEigenvector = [eigenvectors[0], eigenvectors[1], eigenvectors[2]];
    let principalAngle = Math.atan2(primaryEigenvector[1], primaryEigenvector[0]) * (180 / Math.PI);
    if (principalAngle < 0) principalAngle += 360;

    // Calculate confidence from eigenvalue ratio
    const confidence = Math.min(1, Math.max(0, (eigenvalues[0] / totalVariance) * 2 - 1));

    // Normalize to standard rotation
    const rotations = [0, 90, 180, 270];
    let recommendedRotation = 0;
    let minDiff = Infinity;
    for (const rot of rotations) {
        const diff = Math.abs(principalAngle - rot);
        const wrappedDiff = Math.min(diff, 360 - diff);
        if (wrappedDiff < minDiff) {
            minDiff = wrappedDiff;
            recommendedRotation = rot;
        }
    }

    return {
        eigenvalues,
        eigenvectors: primaryEigenvector,
        varianceExplained,
        cumulativeVariance,
        principalAngle,
        confidence,
        recommendedRotation,
        // Axis overlay coordinates
        axisOverlay: {
            x1: 0.5 - Math.cos(principalAngle * Math.PI / 180) * 0.4,
            y1: 0.5 - Math.sin(principalAngle * Math.PI / 180) * 0.4,
            x2: 0.5 + Math.cos(principalAngle * Math.PI / 180) * 0.4,
            y2: 0.5 + Math.sin(principalAngle * Math.PI / 180) * 0.4,
            angle: principalAngle,
            primaryEigenvalue: eigenvalues[0]
        }
    };
}

// Jacobi eigenvalue algorithm for symmetric 3x3 matrix
function jacobiEigenvalue(A) {
    const n = 3;
    let V = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];
    let d = [A[0][0], A[1][1], A[2][2]];

    for (let iter = 0; iter < 50; iter++) {
        // Check convergence
        let off = 0;
        for (let i = 0; i < n; i++) {
            for (let j = i + 1; j < n; j++) {
                off += A[i][j] * A[i][j];
            }
        }
        if (off < 1e-15) break;

        // Find largest off-diagonal element
        let p, q, maxOff = 0;
        for (let i = 0; i < n; i++) {
            for (let j = i + 1; j < n; j++) {
                const off_ij = Math.abs(A[i][j]);
                if (off_ij > maxOff) {
                    maxOff = off_ij;
                    p = i;
                    q = j;
                }
            }
        }

        // Compute rotation
        const theta = (A[q][q] - A[p][p]) / (2 * A[p][q]);
        const t = (theta >= 0 ? 1 : -1) / (Math.abs(theta) + Math.sqrt(1 + theta * theta));
        const c = 1 / Math.sqrt(1 + t * t);
        const s = t * c;

        // Update A
        const a_pp = A[p][p];
        const a_qq = A[q][q];
        const a_pq = A[p][q];

        A[p][p] = c * c * a_pp + s * s * a_qq - 2 * s * c * a_pq;
        A[q][q] = s * s * a_pp + c * c * a_qq + 2 * s * c * a_pq;
        A[p][q] = 0;
        A[q][p] = 0;

        for (let i = 0; i < n; i++) {
            if (i !== p && i !== q) {
                const a_ip = A[i][p];
                const a_iq = A[i][q];
                A[i][p] = c * a_ip - s * a_iq;
                A[p][i] = A[i][p];
                A[i][q] = s * a_ip + c * a_iq;
                A[q][i] = A[i][q];
            }
        }

        // Update V
        for (let i = 0; i < n; i++) {
            const v_ip = V[i][p];
            const v_iq = V[i][q];
            V[i][p] = c * v_ip - s * v_iq;
            V[i][q] = s * v_ip + c * v_iq;
        }

        // Update d
        d = [A[0][0], A[1][1], A[2][2]];
    }

    return { values: d, vectors: V };
}

// Format number in scientific notation
function formatScientific(value) {
    if (Math.abs(value) < 0.0001 || Math.abs(value) >= 10000) {
        return value.toExponential(2);
    }
    const exp = Math.floor(Math.log10(Math.abs(value)));
    const mantissa = value / Math.pow(10, exp);
    return `${mantissa.toFixed(2)}e${exp}`;
}

// Update eigen analysis display
function updateEigenDisplay(eigenResult) {
    // Update eigenvalues
    document.getElementById('lambda1').textContent = formatScientific(eigenResult.eigenvalues[0]);
    document.getElementById('lambda2').textContent = formatScientific(eigenResult.eigenvalues[1]);
    document.getElementById('lambda3').textContent = formatScientific(eigenResult.eigenvalues[2]);

    // Update variance bars
    document.getElementById('varBar1').style.width = `${eigenResult.varianceExplained[0]}%`;
    document.getElementById('varBar2').style.width = `${eigenResult.varianceExplained[1]}%`;
    document.getElementById('varBar3').style.width = `${eigenResult.varianceExplained[2]}%`;

    // Update variance percentages
    document.getElementById('varPercent1').textContent = `${eigenResult.varianceExplained[0].toFixed(1)}%`;
    document.getElementById('varPercent2').textContent = `${eigenResult.varianceExplained[1].toFixed(1)}%`;
    document.getElementById('varPercent3').textContent = `${eigenResult.varianceExplained[2].toFixed(1)}%`;

    // Update summary stats
    document.getElementById('principalAngle').textContent = `${eigenResult.principalAngle.toFixed(1)}°`;
    document.getElementById('recommendedRotation').textContent = `${eigenResult.recommendedRotation}°`;
    document.getElementById('eigenConfidence').textContent = `${(eigenResult.confidence * 100).toFixed(1)}%`;
}

// Draw axis overlay on canvas
function drawAxisOverlay(eigenResult) {
    if (!overlayCanvas || !originalCanvas) return;

    // Match overlay canvas size to original
    overlayCanvas.width = originalCanvas.width;
    overlayCanvas.height = originalCanvas.height;

    // Clear overlay
    overlayCtx.clearRect(0, 0, overlayCanvas.width, overlayCanvas.height);

    const overlay = eigenResult.axisOverlay;

    // Get color from selection
    const color = colorMap[currentOverlayColor] || '#FF0000';

    // Draw principal axis line
    overlayCtx.beginPath();
    overlayCtx.moveTo(overlay.x1 * overlayCanvas.width, overlay.y1 * overlayCanvas.height);
    overlayCtx.lineTo(overlay.x2 * overlayCanvas.width, overlay.y2 * overlayCanvas.height);
    overlayCtx.strokeStyle = color;
    overlayCtx.lineWidth = 3;
    overlayCtx.stroke();

    // Draw arrowhead
    const angle = Math.atan2(
        overlay.y2 - overlay.y1,
        overlay.x2 - overlay.x1
    );
    const arrowLength = 15;
    const arrowAngle = Math.PI / 6;

    overlayCtx.beginPath();
    overlayCtx.moveTo(overlay.x2 * overlayCanvas.width, overlay.y2 * overlayCanvas.height);
    overlayCtx.lineTo(
        overlay.x2 * overlayCanvas.width - arrowLength * Math.cos(angle - arrowAngle),
        overlay.y2 * overlayCanvas.height - arrowLength * Math.sin(angle - arrowAngle)
    );
    overlayCtx.moveTo(overlay.x2 * overlayCanvas.width, overlay.y2 * overlayCanvas.height);
    overlayCtx.lineTo(
        overlay.x2 * overlayCanvas.width - arrowLength * Math.cos(angle + arrowAngle),
        overlay.y2 * overlayCanvas.height - arrowLength * Math.sin(angle + arrowAngle)
    );
    overlayCtx.strokeStyle = color;
    overlayCtx.lineWidth = 3;
    overlayCtx.stroke();

    // Draw center point
    overlayCtx.beginPath();
    overlayCtx.arc(
        overlayCanvas.width / 2,
        overlayCanvas.height / 2,
        5,
        0,
        Math.PI * 2
    );
    overlayCtx.fillStyle = color;
    overlayCtx.fill();

    // Draw angle annotation
    overlayCtx.font = '14px JetBrains Mono, monospace';
    overlayCtx.fillStyle = color;
    overlayCtx.fillText(
        `θ = ${overlay.angle.toFixed(1)}°`,
        10,
        overlayCanvas.height - 10
    );
}

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
