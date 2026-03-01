{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-25.11";
  };
  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
    };
    libPath = with pkgs; lib.makeLibraryPath [
      libGL
      libxkbcommon
      wayland
    ];
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [udev pkg-config fontconfig pkg-config wayland cmake clang libclang libdiscid];
      LD_LIBRARY_PATH = libPath;
      LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
    };
  };
}
