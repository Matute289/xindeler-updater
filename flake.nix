{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flakeCompat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    nci = {
      url = "github:90-008/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.parts.follows = "parts";
      inputs.dream2nix.follows = "dream2nix";
      inputs.crane.follows = "crane";
    };
    parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    dream2nix = {
      url = "github:NeuralModder/dream2nix/update-crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane/v0.21.0";
      flake = false;
    };
  };

  outputs = inputs @ {
    parts,
    nci,
    nixpkgs,
    ...
  }: let
    filteredSource = builtins.path {
      name = "xindeler-updater-source";
      path = toString ./.;
      filter = path: type:
        nixpkgs.lib.all
        (n: builtins.baseNameOf path != n)
        [
          ".github"
          ".gitlab"
          ".gitlab-ci.yml"
          "shell.nix"
          "default.nix"
          "flake.lock"
          "flake.nix"
          "TROUBLESHOOTING.md"
          "CONTRIBUTING.md"
          "CHANGELOG.md"
          "CODE_OF_CONDUCT.md"
          "WORKFLOW.md"
          "PACKAGING.md"
          "README.md"
        ];
    };

    makeVoxygenPatcher = pkgs: let
      runtimeLibs = with pkgs; (
        [libxkbcommon udev alsa-lib stdenv.cc.cc.lib libGL vulkan-loader wayland wayland-protocols]
        ++ (with xorg; [libxcb libX11 libXrandr libXi libXcursor])
      );
    in
      pkgs.writeShellScript "voxygen-patch" ''
        echo "making xindeler-voxygen executable"
        chmod +x xindeler-voxygen
        echo "patching xindeler-voxygen dynamic linker"
        ${pkgs.patchelf}/bin/patchelf \
          --set-interpreter "${pkgs.stdenv.cc.bintools.dynamicLinker}" \
          --set-rpath "${nixpkgs.lib.makeLibraryPath runtimeLibs}" \
          xindeler-voxygen
      '';

    makeServerPatcher = pkgs:
      pkgs.writeShellScript "server-cli-patch" ''
        echo "making xindeler-server-cli executable"
        chmod +x xindeler-server-cli
        echo "patching xindeler-server-cli dynamic linker"
        ${pkgs.patchelf}/bin/patchelf \
          --set-interpreter "${pkgs.stdenv.cc.bintools.dynamicLinker}" \
          xindeler-server-cli
      '';
  in
    parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      imports = [nci.flakeModule];
      perSystem = {
        config,
        pkgs,
        ...
      }: let
        outputs = config.nci.outputs;
        voxygenPatcher = makeVoxygenPatcher pkgs;
        serverPatcher = makeServerPatcher pkgs;
        commonMkDerivation = {
          buildInputs = with pkgs; [openssl];
          nativeBuildInputs = with pkgs; [perl pkg-config];
        };
        serverMkDerivation = {
          buildInputs = with pkgs; [sqlite] ++ commonMkDerivation.buildInputs;
          nativeBuildInputs = commonMkDerivation.nativeBuildInputs;
        };
        wrapPatchers = old:
          pkgs.runCommand
          old.name
          {
            meta = old.meta or {};
            passthru =
              (old.passthru or {})
              // {
                unwrapped = old;
              };
            nativeBuildInputs = [pkgs.makeWrapper];
          }
          ''
            cp -rs --no-preserve=mode,ownership ${old} $out
            wrapProgram $out/bin/* \
              --set XINDELER_VOXYGEN_PATCHER ${voxygenPatcher} \
              --set XINDELER_SERVER_CLI_PATCHER ${serverPatcher} \
          '';
        xindelerUpdater = wrapPatchers outputs."xindeler-updater".packages.release;
      in {
        devShells.default = outputs."xindeler-updater".devShell;
        packages.default = xindelerUpdater;
        packages.xindeler-updater = xindelerUpdater;
        packages.xindeler-updater-dev = wrapPatchers outputs."xindeler-updater".packages.dev;
        packages.xindeler-updater-server-dev = outputs."xindeler-updater-server".packages.dev;
        packages.xindeler-updater-server-release = outputs."xindeler-updater-server".packages.release;

        nci.projects."xindeler-updater" = {
          export = true;
          path = filteredSource;
        };

        nci.crates."xindeler-updater" = {
          export = false;
          runtimeLibs = with pkgs;
            [
              libxkbcommon
              vulkan-loader
              wayland
              wayland-protocols
              xorg.libX11
              xorg.libXrandr
              xorg.libXi
              xorg.libXcursor
            ]
            ++ commonMkDerivation.buildInputs;
          depsDrvConfig.mkDerivation = commonMkDerivation;
          drvConfig.mkDerivation = commonMkDerivation;
        };

        nci.crates."xindeler-updater-server" = {
          # Need to reexport since defining runtimeLibs here causes a strange error with tests or clippy
          export = false;
          runtimeLibs = serverMkDerivation.buildInputs;
          depsDrvConfig.mkDerivation = serverMkDerivation;
          drvConfig.mkDerivation = serverMkDerivation;
        };
      };
    };
}
