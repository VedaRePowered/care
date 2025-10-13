{ pkgs ? import <nixpkgs> {} }:    
  with pkgs; mkShell {
	LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
		libGL
		libxkbcommon
		wayland
	];
	NIX_ENFORCE_PURITY = 0;
	shellHook = ''
		exec zsh
	'';
  }
