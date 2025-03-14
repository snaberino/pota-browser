use::std::fs::File;
use::std::io::{BufReader, Read};
use serde::{Serialize, Deserialize};
// use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct FingerprintManager {
	pub os_type: Vec<Vec<String>>,

}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SingleFingerprint {
	pub os_type: String,
}

pub fn load_fingerprint_manger() -> FingerprintManager {
	let file_path = "fingerprints.json";
	let file = File::open(file_path).expect("Unable to open file");
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content).expect("Unable to read file");
	serde_json::from_str(&content).expect("Unable to parse JSON")
}