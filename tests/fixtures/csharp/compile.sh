#!/bin/bash
set -eu
cd /tmp && dotnet new classlib --name Acme.Tools --framework netstandard2.0 --output project > /dev/null
rm project/Class1.cs
cp /work/Acme.Tools.cs project/
cd project && dotnet build -c Release -p:Deterministic=true -p:DebugType=none -p:GenerateDocumentationFile=false > /tmp/build.log 2>&1 || { tail -30 /tmp/build.log; exit 1; }
cp bin/Release/netstandard2.0/Acme.Tools.dll /work/
echo built
