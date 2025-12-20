# CONTRIBUTING GUIDELINES

## Getting Started

If you are thinking of contributing to this project, you can get started by
forking the project. Detailed instructions are as follows.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)`
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

> [!NOTE]
> _Instructions on creating pull requests is explained in_
> _[Submitting a Pull Request](#submitting-a-pull-request) section below._

## Code of Conduct

### Purpose

This Code of Conduct outlines the expected behavior for all participants in the
community. The goal is to foster a positive, inclusive, and respectful
environment where everyone feels welcome and valued.

### Expected Behavior

- **Respect**: Treat everyone with respect, regardless of their background or
  opinions.
- **Inclusivity**: Promote a welcoming and inclusive environment for all.
- **Constructive Communication**: Engage in constructive and respectful
  discussions, avoiding personal attacks or insults.
- **Openness**: Be open to feedback and criticism, and be willing to learn from
  others.
- **Collaboration**: Work together towards the common goal of improving the
  project.

### Prohibited Behavior

- **Harassment**: Any form of harassment, including but not limited to:
  - Sexual harassment
  - Discrimination
  - Threats
  - Bullying
- **Hate Speech**: Language that promotes hatred or discrimination against any group of people.
- **Personal Attacks**: Attacks on a person's character or abilities.
- **Plagiarism**: Claiming credit for work that is not your own.

> [!IMPORTANT]
> This Code of Conduct is a living document and may be updated from time to time.

## Submitting a Pull Request

- Navigate to your forked repository on GitHub.
- Click the "Compare & pull request" button.
- Add a descriptive title and provide details about your changes.
  You can see the PR template at `.github/PULL_REQUEST_TEMPLATE.md` file.
- Submit the pull request.

## Tagging a new version

> [!IMPORTANT]
> New versions are generally tagged after completing milestones, and will
> probably be updated automatically with workflows in the future. The
> instruction below will be useful for developers to know how versions are
> being tagged in the project.

Install package `cargo-workspaces`

```shell
cargo install cargo-workspaces
```

Run `cargo workspaces` or `cargo ws` command

```shell
# major version
cargo ws version major

# minor version
cargo ws version minor

# patch version
cargo ws version patch
```
