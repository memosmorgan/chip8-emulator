use chip8_emulator::disassembler::{
    disassemble_at, disassemble_opcode, disassemble_range, DisassembledInstruction,
};
use chip8_emulator::memory::Memory;

#[test]
fn disassemble_opcode_formats_cls() {
    assert_eq!(disassemble_opcode(0x00E0), "CLS");
}

#[test]
fn disassemble_opcode_formats_jump() {
    assert_eq!(disassemble_opcode(0x1200), "JP 0x0200");
}

#[test]
fn disassemble_opcode_formats_set_register() {
    assert_eq!(disassemble_opcode(0x6A0F), "LD VA, 0x0F");
}

#[test]
fn disassemble_opcode_formats_draw() {
    assert_eq!(disassemble_opcode(0xD125), "DRW V1, V2, 0x5");
}

#[test]
fn disassemble_opcode_formats_fx33() {
    assert_eq!(disassemble_opcode(0xF133), "LD B, V1");
}

#[test]
fn disassemble_opcode_rejects_invalid_5xyn_as_unknown() {
    assert_eq!(disassemble_opcode(0x5121), "UNKNOWN 0x5121");
}

#[test]
fn disassemble_at_reads_memory_opcode() {
    let mut memory = Memory::new();
    memory.write_byte(0x200, 0x61).expect("write high byte");
    memory.write_byte(0x201, 0x01).expect("write low byte");

    let instruction = disassemble_at(&memory, 0x200).expect("disassemble instruction");

    assert_eq!(
        instruction,
        DisassembledInstruction {
            address: 0x200,
            opcode: 0x6101,
            mnemonic: "LD V1, 0x01".to_string(),
        }
    );
}

#[test]
fn disassemble_range_reads_multiple_instructions() {
    let mut memory = Memory::new();
    let bytes = [0x00, 0xE0, 0x61, 0x01, 0x12, 0x02];
    for (offset, byte) in bytes.iter().enumerate() {
        memory
            .write_byte(0x200 + offset as u16, *byte)
            .expect("write program byte");
    }

    let instructions = disassemble_range(&memory, 0x200, 3).expect("disassemble range");

    assert_eq!(instructions.len(), 3);
    assert_eq!(instructions[0].mnemonic, "CLS");
    assert_eq!(instructions[1].mnemonic, "LD V1, 0x01");
    assert_eq!(instructions[2].mnemonic, "JP 0x0202");
    assert_eq!(instructions[0].address, 0x200);
    assert_eq!(instructions[1].address, 0x202);
    assert_eq!(instructions[2].address, 0x204);
}
