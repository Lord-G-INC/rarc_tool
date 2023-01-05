mod rarc;
mod seektask;


fn main() {
    let mut data = std::fs::read("AbekobeGalaxyMap.arc").unwrap_or_default();
    data = yaz0rust::decompress(&data);
    let rarc = rarc::RARC::read(data);
    rarc.extract();
}
