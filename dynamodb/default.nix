{ pkgs }:
let
 name = "dynamodb";
 dynamo-jar = pkgs.stdenv.mkDerivation {
  name = name;

  src = pkgs.fetchurl {
   url = "https://s3-us-west-2.amazonaws.com/dynamodb-local/dynamodb_local_test_2018-03-23/dynamodb_local_latest.tar.gz";
   sha256 = "0wzp07wdmay4kdc31fs14rbpwch0ncq6zsl7yl3vfa0rk9klgx9x";
  };

  nativeBuildInputs = [ pkgs.makeWrapper ];

  unpackPhase = "tar -zxvf $src";

  installPhase =
  ''
  pwd
  mkdir -p $out/lib
  mv ./DynamoDBLocal_lib $out/DynamoDBLocal_lib
  mv ./DynamoDBLocal.jar $out
  '';
 };

 script = pkgs.writeShellScriptBin "dynamodb"
 ''
 cd ${dynamo-jar}
 mkdir -p $TMP/dynamodb
 ${pkgs.jdk}/bin/java -Djava.library.path=./DynamoDBLocal_lib/ -jar ./DynamoDBLocal.jar -dbPath "$TMP/dynamodb" "$@"
 '';

 inMemory = pkgs.writeShellScriptBin "dynamodb-memory"
 ''
  cd ${dynamo-jar}
  ${pkgs.jdk}/bin/java -Djava.library.path=./DynamoDBLocal_lib/ -jar ./DynamoDBLocal.jar -inMemory "$@"
  '';
in
{
 buildInputs = [ pkgs.jdk script inMemory ];
}
