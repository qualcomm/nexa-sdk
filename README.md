## Directory Structure

- All libraries and tools intended for external release are placed in the repository root
- Dependencies are managed through the build system rather than by directory layout
- Use a single unified build system to avoid complexity caused by different subprojects using different build tools

## Bazel

- Advantages
  - Supports C++/Go/Python/Android and can package Docker images
  - Integrates lint/test/build, so CI only needs to run Bazel commands; local development can also use Bazel to maintain consistency
  - Uses a self-maintained toolchain, unaffected by the system environment
  - Easier to integrate custom build rules compared to CMake
- Disadvantages
  - May be overly complex for small projects (but our project spans multiple languages and platforms)
  - Steeper learning curve (there are plenty of resources; AI can help)

### Common Commands

- `bazel query //...` list all targets in the workspace
- `bazel run //geniex-cli/cmd/geniex-cli` build and run the geniex-cli target, will not build full project.

#### go tips

- `bazel run //gazelle` update BUILD files from `go.mod`
- `# gazelle:resolve go ://example.com //local/foo:go_default_library` resolve external go module to local bazel target

## files

```
.
├── .github                   #
│   ├── actions               # reusable actions, like env setup and s3 config
│   │   ├── env.yml           #
│   │   └── s3.yml            #
│   ├── scripts               # reusable scripts used by workflows, like create github release
│   │   └── release.js        #
│   └── workflows             # github actions workflows, it's simple because we use bazel for everything
│       ├── build.yml         #
│       ├── lint.yml          #
│       └── test.yml          #
│                             #
├── assets                    # assets files for release, doc, etc
│   └── favicon.ico           #
│                             #
├── geniex                    # core library
│   └── BUILD.bazel           #
│                             #
├── geniex-cli                # command line tool
│   ├── cmd                   #
│   │   └── geniex-cli        #
│   │       ├── BUILD.bazel   #
│   │       └── main.go       #
│   ├── internal              #
│   ├── release               #
│   │   ├── docker.bazel      #
│   │   └── msi.bazel         #
│   ├── server                #
│   ├── BUILD.bazel           #
│   ├── go.mod                #
│   └── go.sum                #
│                             #
├── geniex-proc               # pre/post processing library
│   └── BUILD.bazel           #
│                             #
├── geniex-sdk                # sdk for developers
│   ├── build                 #
│   │   ├── llama.cpp.bazel   # build rules for llama.cpp third-party dependency
│   │   └── opencl.bazel      # build rules for opencl third-party dependency
│   ├── include               #
│   ├── libs                  # pre-built library for different platforms
│   ├── src                   #
│   │   ├── ml.cpp            #
│   │   └── plugins           # plugin source code
│   │       ├── geniex        #
│   │       └── llama.cpp     #
│   ├── third-party           # third-party dependencies
│   │   ├── llama.cpp         #
│   │   └── opencl            #
│   └── BUILD.bazel           #
│                             #
├── geniex-sdk-bindings       # language bindings for geniex-sdk
│   ├── android               #
│   │   └── BUILD.bazel       #
│   ├── go                    #
│   │   ├── BUILD.bazel       #
│   │   ├── go.mod            #
│   │   └── ml.go             #
│   └── python                #
│       └── BUILD.bazel       #
│                             #
├── BUILD.bazel               # root BUILD file
├── MODULE.bazel              # root MODULE file
├── MODULE.bazel.lock         #
└── README.md                 #
```

## Notes

- should we keep `dlopen` plugins? there are only two plugins now, and both exist on windows/linux arm64.
- since we will open source all the code, maybe we can static link everything for simplicity.
