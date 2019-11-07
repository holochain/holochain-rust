let
  # Insert your access key here
  accessKey = "AKIAJPZSW4AZW4K4TNBQ";
in {
  # Mapping of our 'trycpserver' machine
  trycpserver = { resources, ... }:
    { deployment.targetEnv = "ec2";
      # We'll be deploying a micro instance to Virginia
      deployment.ec2.region = "us-east-1";
      deployment.ec2.instanceType = "t1.micro";
      deployment.ec2.accessKeyId = accessKey;
      # We'll let NixOps generate a keypair automatically
      deployment.ec2.keyPair = resources.ec2KeyPairs.trycpserver-kp.name;
      # This should be the security group we just created
      deployment.ec2.securityGroups = [ "holochain-test" ];
    };

  # Here we create a keypair in the same region as our deployment
  resources.ec2KeyPairs.trycpserver-kp = {
    region = "us-east-1";
    accessKeyId = accessKey;
  };
}
