# 🤝 Contributing to PCA Compressor

<div align="center">

  **We love contributions!** Here's how you can help make PCA Compressor even better.

  [![PRs Welcome](https://img.shields.io/badge/PRs-Welcome-brightgreen?style=for-the-badge)](https://github.com/kushalchalla981-tech/mathproj/pulls)
  [![Issues Open](https://img.shields.io/badge/Issues-Open-yellow?style=for-the-badge)](https://github.com/kushalchalla981-tech/mathproj/issues)

</div>

---

## 🚀 Getting Started

<div align="center">

### 1️⃣ Fork & Clone

```bash
# Fork the repository
# Click 'Fork' at https://github.com/kushalchalla981-tech/mathproj

# Clone your fork
git clone https://github.com/YOUR_USERNAME/mathproj.git
cd mathproj
```

### 2️⃣ Set Up Development Environment

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
cd pca-compressor
cargo build --all
```

### 3️⃣ Choose an Issue

```bash
# Browse open issues
# Pick one that interests you
# Comment "I'm working on this!" to avoid duplication
```

</div>

---

## 📋 Code Style Guidelines

<div align="center">

### 🎨 Formatting

```bash
# Format all code
cargo fmt

# Check formatting
cargo fmt --check
```

### 🧹 Linting

```bash
# Run clippy
cargo clippy --all-targets

# With warnings as errors
cargo clippy --all-targets -- -D warnings
```

### ✅ Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test pca::test

# Run tests with output
cargo test -- --nocapture

# Run benchmarks (requires --ignore or nightly)
cargo test --ignored
```

</div>

---

## 🛠️ Development Workflow

<div align="center">

```mermaid
graph LR
    A[Fork Repo] --> B[Create Branch]
    B --> C[Make Changes]
    C --> D[Run Tests]
    D --> E[Commit]
    E --> F[Push]
    F --> G[Create PR]
    G --> H[✅ Merged]
```

### Step-by-Step Process

#### 1️⃣ Create a Feature Branch

```bash
git checkout -b feature/your-amazing-feature
```

#### 2️⃣ Make Your Changes

- Write clean, well-documented code
- Add tests for new functionality
- Update relevant documentation
- Follow Rust naming conventions

#### 3️⃣ Test Your Changes

```bash
# Ensure all tests pass
cargo test

# Check formatting
cargo fmt --check

# Run linter
cargo clippy --all-targets
```

#### 4️⃣ Commit Your Changes

```bash
git add .
git commit -m "Add amazing feature: description"
```

**Commit Message Format:**
```
<type>: <subject>

<body>

<footer>
```

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

#### 5️⃣ Push to Your Fork

```bash
git push origin feature/your-amazing-feature
```

#### 6️⃣ Create Pull Request

- Go to https://github.com/kushalchalla981-tech/mathproj
- Click "Compare & pull request"
- Provide a clear description of your changes
- Reference any related issues using `#issue_number`

</div>

---

## 📝 Coding Standards

### Rust

<div align="center">

#### Naming Conventions

```rust
// Types: PascalCase
struct ImageData {}
enum CompressionMode {}
trait Compressor {}

// Functions: snake_case
fn compress_image() {}
fn calculate_ssim() {}

// Constants: SCREAMING_SNAKE_CASE
const MAX_TILE_SIZE: u32 = 2048;

// Variables: snake_case
let image_data = vec![];
let quality_score = 0.7;
```

#### Documentation

```rust
/// Compress an image using PCA compression
///
/// # Arguments
///
/// * `image`: The image to compress
/// * `params`: Compression parameters
///
/// # Returns
///
/// Returns the compressed image and quality metrics
///
/// # Errors
///
/// Returns an error if the image is too small or memory is exceeded
///
/// # Examples
///
/// ```
/// use pca_core::prelude::*;
///
/// let image = load_image("input.jpg")?;
/// let result = compress(&image, &params)?;
/// ```
pub fn compress(image: &ImageData, params: &CompressionParams) -> Result<ImageData> {
    // Implementation
}
```

### Python

#### Style Guide

```python
# PEP 8 compliance
from pca_compressor import compress_file, CompressionParams

def process_image(input_path, output_path, quality=0.7):
    """Process a single image with PCA compression."""
    params = CompressionParams(quality=quality)
    result = compress_file(input_path, output_path, params)
    return result
```

</div>

---

## 🧪 Testing Guidelines

### Test Structure

<div align="center">

```rust
#[cfg(test)]
mod tests {
    // Unit tests go here
    #[test]
    fn test_pca_computation() {
        // Test PCA algorithm correctness
    }

    #[test]
    fn test_ssim_calculation() {
        // Test SSIM metric
    }
}
```

### Coverage Goals

- **Unit Tests**: 80%+ coverage of core logic
- **Integration Tests**: Full pipeline scenarios
- **Edge Cases**: Small images, memory limits, errors

### Performance Tests

```bash
# Run benchmarks (requires nightly toolchain)
cargo +nightly bench
```

</div>

---

## 🎯 Areas Where We Need Help

<div align="center">

### 🚀 High Priority

| Area | Description | Difficulty |
|------|-------------|------------|
| 🧪 Tests | Increase test coverage | 🟢 Easy |
| 📚 Documentation | API docs, guides | 🟢 Easy |
| 🐛 Bug Fixes | Address open issues | 🟡 Medium |
| ✨ Features | New compression modes | 🔴 Hard |

### 📋 Medium Priority

| Area | Description | Difficulty |
|------|-------------|------------|
| 🌐 WebAssembly | WASM compilation | 🟡 Medium |
| 📱 Mobile Apps | iOS/Android native apps | 🔴 Hard |
| 🧵 GPU Acceleration | CUDA/OpenCL integration | 🔴 Hard |
| 🔌 Plugins | Plugin architecture | 🟡 Medium |

### 🌟 Nice to Have

| Area | Description | Difficulty |
|------|-------------|------------|
| 🎨 UI Improvements | GUI enhancement | 🟢 Easy |
| 📊 Visualization | Compression visualizations | 🟡 Medium |
| 🔧 Benchmarking | Performance benchmarks | 🟢 Easy |
| 🌍 Internationalization | Multi-language support | 🟡 Medium |

</div>

---

## 📏 Pull Request Checklist

<div align="center">

### ✨ Before Submitting

- [ ] Code follows style guidelines
- [ ] All tests pass (`cargo test`)
- [ ] Documentation updated
- [ ] Commit messages follow format
- [ ] No merge conflicts
- [ ] Ready for review

### 📝 PR Description Template

```markdown
## Description
Brief description of your changes

## Changes
- Bullet points of what you changed
- Links to related issues

## Testing
- Tests added: `cargo test my_feature`
- Manual testing steps

## Screenshots
(if UI changes)
```

</div>

---

## 🏆 Recognition

<div align="center">

### Contributors will be recognized in:

- 📚 README.md contributors section
- 🏅 GitHub contributor stats
- 🎉 Release changelogs
- ⭐ Special issue acknowledgments

### Top Contributors

(Will be populated as contributions grow)

| Contributor | Commits | PRs |
|-------------|---------|-----|
| Coming soon! | - | - |

</div>

---

## 💬 Communication

<div align="center">

### 📧 For Questions

- **GitHub Issues**: https://github.com/kushalchalla981-tech/mathproj/issues
- **GitHub Discussions**: https://github.com/kushalchalla981-tech/mathproj/discussions
- **Email**: kushalchalla981@gmail.com

### 🏷️ Labels

- `good first issue` - Great for newcomers
- `help wanted` - Community help needed
- `enhancement` - New features
- `bug` - Bug fixes
- `documentation` - Docs improvements
- `performance` - Performance optimizations

### 🎉 Getting Started Tips

1. **Start Small**: Pick a "good first issue"
2. **Ask Questions**: Don't hesitate to reach out
3. **Learn from Others**: Review PRs and code
4. **Stay Active**: Engage in discussions

---

## 📄 License

By contributing, you agree that your contributions will be licensed under the **MIT License**.

---

<div align="center">

### 🙏 Thank You for Your Contribution!

<div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 20px; border-radius: 15px; color: white;">

**"Alone we can do so little; together we can do so much."**
— Helen Keller

</div>

---

<div align="center">

[![forthebadge](https://forthebadge.com/images/badges/built-with-love.svg)](https://github.com/kushalchalla981-tech/mathproj)
[![forthebadge](https://forthebadge.com/images/badges/developers-die-hard.svg)](https://github.com/kushalchalla981-tech/mathproj)

</div>