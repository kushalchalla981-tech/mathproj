name: Feature Request
description: Suggest a new feature or enhancement
labels: [enhancement, needs-discussion]
body:
  - type: markdown
    attributes:
      value: |
        ## ✨ Feature Request

        Have an idea for a new feature? We'd love to hear it!

  - type: textarea
    id: summary
    attributes:
      label: Summary
      description: A brief summary of the feature
      placeholder: A concise summary of the feature request
    validations:
      required: true

  - type: textarea
    id: motivation
    attributes:
      label: Motivation
      description: What problem does this solve? Who would benefit?
      placeholder: |
        Explain the motivation behind this feature:
        - What problem does it solve?
        - Who would benefit from this feature?
        - What use cases does it enable?
    validations:
      required: true

  - type: textarea
    id: solution
    attributes:
      label: Proposed Solution
      description: How should we solve this problem?
      placeholder: |
        Describe your proposed solution:
        - How should it work?
        - What API changes are needed (if any)?
        - Include code snippets or pseudo-code if helpful
    validations:
      required: true

  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives Considered
      description: Any alternative approaches you've considered
      placeholder: |
        Describe any alternative solutions you've considered:
        - What are the trade-offs?
        - Why did you choose this approach?
    validations:
      required: false

  - type: textarea
    id: mathematics
    attributes:
      label: Mathematical Background (if applicable)
      description: For algorithm-related features, provide mathematical context
      placeholder: |
        For algorithm features, include relevant mathematical formulas:
        - Eigenvalue/Eigenvector equations
        - Matrix operations
        - Statistical formulas
      render: latex

  - type: dropdown
    id: component
    attributes:
      label: Target Component
      description: Which part of the project should implement this?
      options:
        - Core Library (pca-core)
        - CLI (pca-cli)
        - Python Bindings (pca-py)
        - Desktop GUI (pca-gui)
        - Web Interface
        - Documentation
        - Multiple components
        - Not sure

  - type: dropdown
    id: priority
    attributes:
      label: Priority
      description: How important is this feature?
      options:
        - High - Critical for project goals
        - Medium - Important but not critical
        - Low - Nice to have
        - None - Just an idea

  - type: textarea
    id: mockups
    attributes:
      label: Mockups / Wireframes
      description: Any UI mockups or design ideas
      placeholder: Attach or link to any design mockups
