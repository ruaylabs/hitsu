{
  description = "Hitsu desktop password manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      devPkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      hitsu = pkgs.callPackage ./nix/package.nix { };
    in
    {
      packages.${system} = {
        inherit hitsu;
        default = hitsu;
      };

      apps.${system}.default = {
        type = "app";
        program = "${self.packages.${system}.hitsu}/bin/hitsu";
      };

      devShells.${system}.default = devPkgs.callPackage ./nix/shell.nix { };

      checks.${system}.hitsu = hitsu;
      formatter.${system} = pkgs.nixfmt-rfc-style;
    };
}
