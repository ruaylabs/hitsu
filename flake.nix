{
  description = "Hitsu desktop password manager";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
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

      checks.${system}.hitsu = hitsu;
      formatter.${system} = pkgs.nixfmt-rfc-style;
    };
}
