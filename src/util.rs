use szs;
use szs::EncodeAlgo;

pub fn decode<A: AsRef<[u8]>>(src: A) -> Result<Vec<u8>, szs::Error> {
    szs::decode(src.as_ref())
}

pub fn encode<A: AsRef<[u8]>>(src: A, algo: EncodeAlgo) -> Result<Vec<u8>, szs::Error> {
    szs::encode(src.as_ref(), algo)
}