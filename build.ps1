cargo b --release
$TARGETDIR = "build"
if(Test-Path -Path $TARGETDIR) {
  Remove-Item -Path $TARGETDIR -Recurse
}
if(!(Test-Path -Path $TARGETDIR)) {
  New-Item -ItemType Directory -Path $TARGETDIR
}
Copy-Item -Path "target/release/rust_table_export_simple.exe" -Destination "$TARGETDIR/export_table.exe"
Copy-Item -Recurse -Path "templates" -Destination $TARGETDIR
Copy-Item -Recurse -Path "access_example" -Destination $TARGETDIR