import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';

export class ValheimStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // looking up default VPC
    const vpc = ec2.Vpc.fromLookup(this, 'Valheim-VPC', { isDefault: true });

    // looking up an SG by its ID
    const sg = ec2.SecurityGroup.fromSecurityGroupId(this, 'Valheim-SG', 'sg-a975eaf8')

    // creating the EC2 instance
    const instance = new ec2.Instance(this, 'DR-Instance', {
      vpc: vpc,
      securityGroup: sg,
      instanceName: 'Valheim',
      keyName: 'valheim',
      instanceType: new ec2.InstanceType('t3.medium'),
      machineImage: new ec2.AmazonLinuxImage()
    });

    const out = new cdk.CfnOutput(this, "Dungeon-IP", {
      value: instance.instancePublicIp
    });

  }
}
