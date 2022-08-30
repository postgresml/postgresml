# Table of Contents

1. [Contributing to CUB](#contributing-to-cub)
1. [CMake Options](#cmake-options)
1. [Development Model](#development-model)

# Contributing to CUB

CUB uses Github to manage all open-source development, including bug tracking,
pull requests, and design discussions. This document details how to get
started as a CUB contributor.

An overview of this process is:

1. [Clone the CUB repository](#clone-the-cub-repository)
1. [Setup a fork of CUB](#setup-a-fork-of-cub)
1. [Setup your environment](#setup-your-environment)
1. [Create a development branch](#create-a-development-branch)
1. [Local development loop](#local-development-loop)
1. [Push development branch to your fork](#push-development-branch-to-your-fork)
1. [Create pull request](#create-pull-request)
1. [Address feedback and update pull request](#address-feedback-and-update-pull-request)
1. [When your PR is approved...](#when-your-pr-is-approved)

## Clone the CUB Repository

To get started, clone the main repository to your local computer:

```
git clone https://github.com/NVIDIA/cub.git
cd cub
```

## Setup a Fork of CUB

You'll need a fork of CUB on Github to create a pull request. To setup your
fork:

1. Create a Github account (if needed)
2. Go to [the CUB Github page](https://github.com/NVIDIA/cub)
3. Click "Fork" and follow any prompts that appear.

Once your fork is created, setup a new remote repo in your local CUB clone:

```
git remote add github-fork git@github.com:<GITHUB_USERNAME>/cub.git
```

## Setup Your Environment

### Git Environment

If you haven't already, this is a good time to tell git who you are. This
information is used to fill out authorship information on your git commits.

```
git config --global user.name "John Doe"
git config --global user.email johndoe@example.com
```

### Configure CMake builds

CUB uses [CMake](https://www.cmake.org) for its developer build system. To
configure, build, and test your checkout of CUB with default settings:

```
# Create build directory:
mkdir build
cd build

# Configure -- use one of the following:
cmake ..   # Command line interface.
ccmake ..  # ncurses GUI (Linux only)
cmake-gui  # Graphical UI, set source/build directories in the app

# Build:
cmake --build . -j <num jobs>   # invokes make (or ninja, etc)

# Run tests and examples:
ctest
```

See [CMake Options](#cmake-options) for details on customizing the build.

## Create a Development Branch

All work should be done in a development branch (also called a "topic branch")
and not directly in the `main` branch. This makes it easier to manage multiple
in-progress patches at once, and provides a descriptive label for your patch
as it passes through the review system.

To create a new branch based on the current `main`:

```
# Checkout local main branch:
cd /path/to/cub/sources
git checkout main

# Sync local main branch with github:
git pull

# Create a new branch named `my_descriptive_branch_name` based on main:
git checkout -b my_descriptive_branch_name

# Verify that the branch has been created and is currently checked out:
git branch
```

CUB branch names should follow a particular pattern:

- For new features, name the branch `feature/<name>`
- For bugfixes associated with a github issue, use `bug/github/<bug-description>-<bug-id>`
  - Internal nvidia and gitlab bugs should use `nvidia` or `gitlab` in place of
    `github`.

## Local Development Loop

### Edit, Build, Test, Repeat

Once the topic branch is created, you're all set to start working on CUB
code. Make some changes, then build and test them:

```
# Implement changes:
cd /path/to/cub/sources
emacs cub/some_file.cuh # or whatever editor you prefer

# Create / update a unit test for your changes:
emacs tests/some_test.cu

# Check that everything builds and tests pass:
cd /path/to/cub/build/directory
cmake --build . -j <num_jobs> # or make, ninja, etc
ctest
```

### Creating a Commit

Once you're satisfied with your patch, commit your changes:

```
# Manually add changed files and create a commit:
cd /path/to/cub
git add cub/some_file.cuh
git add tests/some_test.cu
git commit

# Or, if possible, use git-gui to review your changes while building your patch:
git gui
```

#### Writing a Commit Message

Your commit message will communicate the purpose and rationale behind your
patch to other developers, and will be used to populate the initial description
of your Github pull request.

When writing a commit message, the following standard format should be used,
since tools in the git ecosystem are designed to parse this correctly:

```
First line of commit message is a short summary (<80 char)
<Second line left blank>
Detailed description of change begins on third line. This portion can
span multiple lines, try to manually wrap them at something reasonable.

Blank lines can be used to separate multiple paragraphs in the description.

If your patch is associated with another pull request or issue in the main
CUB repository, you should reference it with a `#` symbol, e.g.
#1023 for issue 1023.

For issues / pull requests in a different github repo, reference them using
the full syntax, e.g. NVIDIA/thrust#4 for issue 4 in the NVIDIA/thrust repo.

Markdown is recommended for formatting more detailed messages, as these will
be nicely rendered on Github, etc.
```

## Push Development Branch to your Fork

Once you've committed your changes to a local development branch, it's time to
push them to your fork:

```
cd /path/to/cub/checkout
git checkout my_descriptive_branch_name # if not already checked out
git push --set-upstream github-fork my_descriptive_branch_name
```

`--set-upstream github-fork` tells git that future pushes/pulls on this branch
should target your `github-fork` remote by default.

## Create Pull Request

To create a pull request for your freshly pushed branch, open your github fork
in a browser by going to `https://www.github.com/<GITHUB_USERNAME>/cub`. A
prompt may automatically appear asking you to create a pull request if you've
recently pushed a branch.

If there's no prompt, go to "Code" > "Branches" and click the appropriate
"New pull request" button for your branch.

If you would like a specific developer to review your patch, feel free to
request them as a reviewer at this time.

The CUB team will review your patch, test it on NVIDIA's internal CI, and
provide feedback.

## Address Feedback and Update Pull Request

If the reviewers request changes to your patch, use the following process to
update the pull request:

```
# Make changes:
cd /path/to/cub/sources
git checkout my_descriptive_branch_name
emacs cub/some_file.cuh
emacs tests/some_test.cu

# Build + test
cd /path/to/thrust/build/directory
cmake --build . -j <num jobs>
ctest

# Amend commit:
cd /path/to/cub/sources
git add cub/some_file.cuh
git add tests/some_test.cu
git commit --amend
# Or
git gui # Check the "Amend Last Commit" box

# Update the branch on your fork:
git push -f
```

At this point, the pull request should show your recent changes.

## When Your PR is Approved

Once your pull request is approved by the CUB team, no further action is
needed from you. We will handle integrating it since we must coordinate changes
to `main` with NVIDIA's internal perforce repository.

# CMake Options

A CUB build is configured using CMake options. These may be passed to CMake
using

```
cmake -D<option_name>=<value> /path/to/cub/sources
```

or configured interactively with the `ccmake` or `cmake-gui` interfaces.

The configuration options for CUB are:

- `CMAKE_BUILD_TYPE={Release, Debug, RelWithDebInfo, MinSizeRel}`
  - Standard CMake build option. Default: `RelWithDebInfo`
- `CUB_ENABLE_HEADER_TESTING={ON, OFF}`
  - Whether to test compile public headers. Default is `ON`.
- `CUB_ENABLE_TESTING={ON, OFF}`
  - Whether to build unit tests. Default is `ON`.
- `CUB_ENABLE_EXAMPLES={ON, OFF}`
  - Whether to build examples. Default is `ON`.
- `CUB_ENABLE_DIALECT_CPPXX={ON, OFF}`
  - Toggle whether a specific C++ dialect will be targeted.
  - Multiple dialects may be targeted in a single build.
  - Possible values of `XX` are `{11, 14, 17}`.
  - By default, only C++14 is enabled.
- `CUB_ENABLE_COMPUTE_XX={ON, OFF}`
  - Controls the targeted CUDA architecture(s)
  - Multiple options may be selected when using NVCC as the CUDA compiler.
  - Valid values of `XX` are:
    `{35, 37, 50, 52, 53, 60, 61, 62, 70, 72, 75, 80}`
  - Default value depends on `CUB_DISABLE_ARCH_BY_DEFAULT`:
- `CUB_ENABLE_COMPUTE_FUTURE={ON, OFF}`
  - If enabled, CUDA objects will target the most recent virtual architecture
    in addition to the real architectures specified by the
    `CUB_ENABLE_COMPUTE_XX` options.
  - Default value depends on `CUB_DISABLE_ARCH_BY_DEFAULT`:
- `CUB_DISABLE_ARCH_BY_DEFAULT={ON, OFF}`
  - When `ON`, all `CUB_ENABLE_COMPUTE_*` options are initially `OFF`.
  - Default: `OFF` (meaning all architectures are enabled by default)
- `CUB_ENABLE_TESTS_WITH_RDC={ON, OFF}`
  - Whether to enable Relocatable Device Code when building tests.
    Default is `OFF`.
- `CUB_ENABLE_EXAMPLES_WITH_RDC={ON, OFF}`
  - Whether to enable Relocatable Device Code when building examples.
    Default is `OFF`.
- `CUB_ENABLE_INSTALL_RULES={ON, OFF}`
  - If true, installation rules will be generated for CUB. Default is `ON` when
    building CUB alone, and `OFF` when CUB is a subproject added via CMake's
    `add_subdirectory`.

# Development Model

The following is a description of the basic development process that CUB follows. This is a living
document that will evolve as our process evolves.

CUB is distributed in three ways:

   * On GitHub.
   * In the NVIDIA HPC SDK.
   * In the CUDA Toolkit.

## Trunk Based Development

CUB uses [trunk based development](https://trunkbaseddevelopment.com). There is a single long-lived
branch called `main`. Engineers may create branches for feature development. Such branches always
merge into `main`. There are no release branches. Releases are produced by taking a snapshot of
`main` ("snapping"). After a release has been snapped from `main`, it will never be changed.

## Repositories

As CUB is developed both on GitHub and internally at NVIDIA, there are three main places where code lives:

   * The Source of Truth, the [public CUB repository](https://github.com/NVIDIA/cub), referred to as
     `github` later in this document.
   * An internal GitLab repository, referred to as `gitlab` later in this document.
   * An internal Perforce repository, referred to as `perforce` later in this document.

## Versioning

CUB has its own versioning system for releases, independent of the versioning scheme of the NVIDIA
HPC SDK or the CUDA Toolkit.

Today, CUB version numbers have a specific [semantic meaning](https://semver.org/).
Releases prior to 1.10.0 largely, but not strictly, followed these semantic meanings.

The version number for a CUB release uses the following format: `MMM.mmm.ss-ppp`, where:

   * `CUB_VERSION_MAJOR`/`MMM`: Major version, up to 3 decimal digits. It is incremented
     when the fundamental nature of the library evolves, leading to widespread changes across the
     entire library interface with no guarantee of API, ABI, or semantic compatibility with former
     versions.
   * `CUB_VERSION_MINOR`/`mmm`: Minor version, up to 3 decimal digits. It is incremented when
     breaking API, ABI, or semantic changes are made.
   * `CUB_VERSION_SUBMINOR`/`ss`: Subminor version, up to 2 decimal digits. It is incremented
     when notable new features or bug fixes or features that are API, ABI, and semantic backwards
     compatible are added.
   * `CUB_PATCH_NUMBER`/`ppp`: Patch number, up to 3 decimal digits. It is incremented if any
     change in the repo whatsoever is made and no other version component has been incremented.

The `<cub/version.h>` header defines `CUB_*` macros for all of the version components mentioned
above. Additionally, a `CUB_VERSION` macro is defined, which is an integer literal containing all
of the version components except for `CUB_PATCH_NUMBER`.

## Branches and Tags

The following tag names are used in the CUB project:

  * `github/nvhpc-X.Y`: the tag that directly corresponds to what has been shipped in the NVIDIA HPC SDK release X.Y.
  * `github/cuda-X.Y`: the tag that directly corresponds to what has been shipped in the CUDA Toolkit release X.Y.
  * `github/A.B.C`: the tag that directly corresponds to CUB version A.B.C.
  * `github/A.B.C-rcN`: the tag that directly corresponds to CUB version A.B.C release candidate N.

The following branch names are used in the CUB project:

  * `github/main`: the Source of Truth development branch of CUB.
  * `github/old-master`: the old Source of Truth branch, before unification of public and internal repositories.
  * `github/feature/<name>`: feature branch for a feature under development.
  * `github/bug/<bug-system>/<bug-description>-<bug-id>`: bug fix branch, where `bug-system` is `github` or `nvidia`.
  * `gitlab/main`: mirror of `github/main`.
  * `perforce/private`: mirrored `github/main`, plus files necessary for internal NVIDIA testing systems.

On the rare occasion that we cannot do work in the open, for example when developing a change specific to an
unreleased product, these branches may exist on `gitlab` instead of `github`. By default, everything should be
in the open on `github` unless there is a strong motivation for it to not be open.
