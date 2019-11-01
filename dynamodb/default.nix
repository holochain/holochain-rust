{ pkgs }:
let
 name = "dynamodb";
 dynamo-jar = pkgs.stdenv.mkDerivation {
  name = name;

  src = pkgs.fetchurl {
   url = "https://s3-us-west-2.amazonaws.com/dynamodb-local/dynamodb_local_2019-02-07.tar.gz";
   sha256 = "0hrwxg4igyll40y7l1s0icg55g247fl8cjs4rrcpjf8d7m0bb09j";
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

 memory = pkgs.writeShellScriptBin "dynamodb-memory"
 ''
 cd ${dynamo-jar}
 ${pkgs.jdk}/bin/java -Djava.library.path=./DynamoDBLocal_lib/ -jar ./DynamoDBLocal.jar -inMemory "$@"
 '';
in
{
 buildInputs = [ pkgs.jdk script memory ];
}
