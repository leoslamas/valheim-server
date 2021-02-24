#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from '@aws-cdk/core';
import { ValheimStack } from '../lib/cdk-stack';

const env: cdk.Environment = { account: '243101742269', region: 'us-east-1' };
const app = new cdk.App();
new ValheimStack(app, 'ValheimStack', { env: env });
