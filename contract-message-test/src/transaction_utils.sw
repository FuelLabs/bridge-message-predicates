library transaction_utils;

// TODO: replace GTF consts with direct references to tx.sw, inputs.sw, and outputs.sw from std lib
const GTF_INPUT_MESSAGE_DATA_LENGTH = 0x11B;
const GTF_INPUT_MESSAGE_DATA = 0x11E;

/// Get the length of a message input data
pub fn input_message_data_length(index: u64) -> u64 {
    __gtf::<u64>(index, GTF_INPUT_MESSAGE_DATA_LENGTH)
}

/// Get the data of a message input
pub fn input_message_data<T>(index: u64, offset: u64) -> T {
    // TODO: look into supporting per byte offsets
    let data_ptr = __gtf::<raw_ptr>(index, GTF_INPUT_MESSAGE_DATA);
    data_ptr.add::<u64>(offset / 8).read::<T>()
}
