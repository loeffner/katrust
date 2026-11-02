{
  description = "KatRust: KaTrain reimplemented in Rust.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    git-hooks.url = "github:cachix/git-hooks.nix";
    git-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      git-hooks,
    }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;

      pre-commit-check = git-hooks.lib.x86_64-linux.run {
        src = ./.;
        hooks = {
          nixfmt.enable = true;
          rustfmt.enable = true;
          clippy = {
            enable = true;
            settings.denyWarnings = true;
          };
        };
      };
    in
    {
      checks.x86_64-linux.pre-commit-check = pre-commit-check;

      devShells.x86_64-linux.default = pkgs.mkShell {
        packages = with pkgs; [
          rustc
          cargo
          clippy
          rustfmt
        ];

        inherit (pre-commit-check) shellHook;
      };

      formatter.x86_64-linux = pkgs.nixfmt-tree;
    };
}
