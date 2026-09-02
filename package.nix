{
  rustPlatform,
}:
let
  commonAttrs = {
    src = ./.;
    cargoLock = {
      lockFile = ./Cargo.lock;
    };
  };
  getVersion = (path: (fromTOML (builtins.readFile ./${path}/Cargo.toml)).package.version);
in
{
  anisyncd = rustPlatform.buildRustPackage (
    commonAttrs
    // {
      pname = "anisyncd";
      version = getVersion "anisyncd";
      buildAndTestSubdir = "./anisyncd";
    }
  );

  anisync-cli = rustPlatform.buildRustPackage (
    commonAttrs
    // {
      pname = "anisync-cli";
      version = getVersion "anisync-cli";
      buildAndTestSubdir = "./anisync-cli";
    }
  );
}
