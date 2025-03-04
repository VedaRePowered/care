{ pkgs ? import <nixpkgs> {} }:    
  with pkgs; mkShell {
	LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
		libGL
		libxkbcommon
		wayland
	];
	shellHook = ''
		exec zsh
	'';
  }
