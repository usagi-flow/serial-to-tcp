{
	description = "An app which reads data from a serial port and serves it on a TCP port.";

	inputs = {
		nixpkgs.url = github:NixOS/nixpkgs/nixos-23.05;
		flake-utils.url = "github:numtide/flake-utils";
	};

	outputs = { self, nixpkgs, flake-utils, ... }: flake-utils.lib.eachDefaultSystem (system:
		let
			pkgs = import nixpkgs { inherit system; };
			main-package = (with pkgs; stdenv.mkDerivation {
				pname = "serial-to-tcp";
				version = "1.0.1";
				src = self;
				nativeBuildInputs = [
					rustc
					cargo
				];
				buildPhase = "cargo build";
				installPhase = "mkdir -p $out/bin; install -t $out/bin target/debug/serial-to-tcp";
			});
		in rec {
			defaultApp = flake-utils.lib.mkApp { drv = defaultPackage; };
			defaultPackage = main-package;
		}
	);
}
