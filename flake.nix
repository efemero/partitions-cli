{
  description = "Partitions CLI";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      naersk,
      rust-overlay,
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };

      naerskLib = pkgs.callPackage naersk { };
      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      fontPackages = with pkgs; [
        gyre-fonts
        dejavu_fonts
        liberation_ttf
        freefont_ttf
        unifont
      ];

      ghostscriptFontDir = "${pkgs.ghostscript.fonts}/share/ghostscript/${pkgs.ghostscript.version}/Resource/Font";

      fontsConf = pkgs.makeFontsConf {
        fontDirectories = [ ghostscriptFontDir ] ++ map (font: "${font}/share/fonts") fontPackages;
      };

      lilypondBinPath = pkgs.lib.makeBinPath [ pkgs.lilypond ];
    in
    {
      packages.${system}.default = naerskLib.buildPackage {
        pname = "partitions";
        version = "0.1.0";
        src = ./.;
        cargo = rustToolchain;
        rustc = rustToolchain;
        nativeBuildInputs = with pkgs; [
          pkg-config
          makeWrapper
        ];
        buildInputs = [
          pkgs.fontconfig
          pkgs.ghostscript
        ]
        ++ fontPackages;
        postInstall = ''
          wrapProgram "$out/bin/partitions" \
            --prefix PATH : ${lilypondBinPath} \
            --set FONTCONFIG_FILE ${fontsConf} \
            --set FONTCONFIG_PATH ${builtins.dirOf fontsConf}
        '';
      };

      devShells.${system}.default = pkgs.mkShell {
        buildInputs =
          with pkgs;
          [
            rustToolchain
            pkg-config
            lilypond
            fontconfig
            ghostscript
          ]
          ++ fontPackages;

        nativeBuildInputs = with pkgs; [
          prek
          python313
          python313Packages.jinja2
          uv
        ];
        FONTCONFIG_FILE = fontsConf;
        FONTCONFIG_PATH = builtins.dirOf fontsConf;

        shellHook = ''
          if [ -d "${rustToolchain}/lib/rustlib/src/rust/library" ]; then
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
          fi
        '';
      };
    };
}
