source = ["./target/release/vmal"]
bundle_id = "com.dotconnor.vmal"

apple_id {
  username = "dotdotconnor@icloud.com"
  password = "@keychain:AC_PASSWORD"
  provider = "GAQQB7DAG2"
}

sign {
  application_identity = "9669CDD3D59D0116A1722E07B8957052EB6497D4"
}

zip {
  output_path = "./dist/vmal.zip"
}