name: Bug Report
description: Report something that is not working correctly
labels: [bug]
body:
  - type: markdown
    attributes:
      value: |
        ## 🐛 Bug Report

        Thank you for reporting a bug! Please help us understand the issue.

  - type: textarea
    id: description
    attributes:
      label: Description
      description: A clear description of the bug
      placeholder: Describe the bug in detail
    validations:
      required: true

  - type: textarea
    id: steps
    attributes:
      label: Steps to Reproduce
      description: How to reproduce this bug
      placeholder: |
        1. Go to '...'
        2. Click on '...'
        3. See error
    validations:
      required: true

  - type: textarea
    id: expected
    attributes:
      label: Expected Behavior
      description: What should happen
      placeholder: Tell us what you expected to happen
    validations:
      required: true

  - type: textarea
    id: actual
    attributes:
      label: Actual Behavior
      description: What actually happens
      placeholder: Tell us what actually happens
    validations:
      required: true

  - type: dropdown
    id: version
    attributes:
      label: Version
      description: Which version are you using?
      options:
        - Latest (main branch)
        - Latest stable release
        - Development version
    validations:
      required: true

  - type: dropdown
    id: component
    attributes:
      label: Component
      description: Which part of the project is affected?
      options:
        - Core Library (pca-core)
        - CLI (pca-cli)
        - Python Bindings (pca-py)
        - Desktop GUI (pca-gui)
        - Web Interface
        - Documentation

  - type: textarea
    id: environment
    attributes:
      label: Environment
      description: Your operating system, architecture, etc.
      placeholder: |
        - OS: [e.g., Ubuntu 22.04, Windows 11, macOS 14]
        - Rust version: [e.g., 1.75.0]
        - Python version: [e.g., 3.11]
      render: shell

  - type: textarea
    id: logs
    attributes:
      label: Relevant Log Output
      description: Any relevant log output or error messages
      placeholder: Paste any relevant error messages here
      render: shell

  - type: textarea
    id: attachments
    attributes:
      label: Attachments
      description: Screenshots, test images, or other relevant files
      placeholder: Drag and drop or paste links here
