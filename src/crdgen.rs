use kube::CustomResourceExt;
fn main() {
    print!("{}", serde_yaml::to_string(&controller::EndpointReference::crd()).unwrap());
    println!("---");
    print!("{}", serde_yaml::to_string(&controller::FrpClient::crd()).unwrap());
}
