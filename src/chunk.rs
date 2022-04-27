pub struct Chunk
{
	pub data: [u8; 1024],
	pub size: usize
}

impl Default for Chunk
{
    fn default() -> Self
	{
        return Chunk {
			data: [0u8; 1024],
			size: 0usize
		}
    }
}
