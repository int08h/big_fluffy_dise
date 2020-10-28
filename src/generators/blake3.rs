use blake3::OutputReader;

/// Generate the contents of a BigKey using Blake3
pub struct Blake3Generator {
    xof: OutputReader,
}
