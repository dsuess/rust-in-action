const NUM_REGISTERS: usize = 16;
const STACKSIZE: usize = 16;
const MEMSIZE: usize = 0x1000;
const ERRFLAG: usize = 0xF;

fn main() {
    let mut cpu = CPU::new();

    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    let mem = &mut cpu.memory;
    // Call "add twice function"
    mem[0x000..0x002].copy_from_slice(&[0x21, 0x00]);
    // "Add twice function"
    mem[0x100..0x106].copy_from_slice(&[0x80, 0x14, 0x80, 0x14, 0x00, 0xEE]);
    cpu.run();

    println!("5 + 10 + 10 = {}", cpu.registers[0]);
}


struct CPU {
    memory: [u8; MEMSIZE],
    registers: [u8; NUM_REGISTERS],
    pcounter: u16,
    // TODO Use stack datatype
    stack: [u16; STACKSIZE],
    stackptr: u16
}


impl CPU {
    pub fn new() -> CPU {
        CPU { memory: [0; MEMSIZE], registers: [0; NUM_REGISTERS], pcounter: 0, stack: [0; STACKSIZE], stackptr: 0 }
    }

    fn read_opcode(&mut self) -> u16 {
        let a = self.memory[self.pcounter as usize];
        let b = self.memory[(self.pcounter + 1) as usize];
        self.pcounter += 2;

        ((a as u16) << 8) | (b as u16)
    }

    fn parse_opcode(opcode: u16) -> (u8, u8, u8, u8, u16) {
        let c = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let d = ((opcode & 0xF00F) >> 0) as u8;
        let nnn = opcode & 0x0FFF;
        (c, x, y, d, nnn)
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.read_opcode();
            let (c, x, y, d, nnn) = CPU::parse_opcode(opcode);
            match (c, x, y, d) {
                (0x8, _, _, 0x4) => self.add_xy(x, y),
                (0x2, _, _, _) => self.call(nnn),
                (_, _, 0xE, 0xE) => self.ret(),
                (0, 0, 0, 0) => break,
                _ => todo!("Unknown opcode {:04x}", opcode),
            }
        }
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        let (val, overflow) = self.registers[x as usize].overflowing_add(self.registers[y as usize]);

        self.registers[x as usize] = val;
        self.registers[ERRFLAG] = overflow as u8;
    }

    fn call(&mut self, nnn: u16) {
        if (self.stackptr as usize)>= STACKSIZE - 1 {
            panic!("Stack Overflow");
        }
        self.stack[self.stackptr as usize] = self.pcounter as u16;
        self.stackptr += 1;
        self.pcounter = nnn;
    }

    fn ret(&mut self) {
        if self.stackptr == 0 {
            panic!("Stack Underflow");
        }
        self.stackptr -= 1;
        self.pcounter = self.stack[self.stackptr as usize];
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition_simple() {
        let mut cpu = CPU::new();
        cpu.memory[0] = 0x80;
        cpu.memory[1] = 0x14;
        cpu.registers[0] = 5;
        cpu.registers[1] = 10;
        cpu.run();
        assert_eq!(cpu.registers[0], 15);
        assert_eq!(cpu.registers[ERRFLAG], 0);
    }

    #[test]
    fn test_addition_overflow() {
        let mut cpu = CPU::new();
        cpu.memory[0] = 0x80;
        cpu.memory[1] = 0x14;
        cpu.registers[0] = 255;
        cpu.registers[1] = 255;
        cpu.run();
        assert_eq!(cpu.registers[0], 254);
        assert_eq!(cpu.registers[ERRFLAG], 1);
    }

    #[test]
    fn test_addition_multiple() {
        let mut cpu = CPU::new();

        cpu.registers[0] = 5;
        for i in (0..6).step_by(2) {
            cpu.memory[i] = 0x80;
            cpu.memory[i + 1] = 0x04 + ((i / 2 + 1) as u8) * 0x10;
            cpu.registers[i / 2 + 1] = 10 + (i as u8);
        }
        cpu.run();

        assert_eq!(cpu.registers[0], 41);
        assert_eq!(cpu.registers[ERRFLAG], 0);
    }

    #[test]
    fn test_funccall() {
        let mut cpu = CPU::new();

        cpu.registers[0] = 5;
        cpu.registers[1] = 10;

        let mem = &mut cpu.memory;
        // Call "add twice function"
        mem[0x000..0x002].copy_from_slice(&[0x21, 0x00]);
        mem[0x002..0x004].copy_from_slice(&[0x21, 0x00]);
        // "Add twice function"
        mem[0x100..0x106].copy_from_slice(&[0x80, 0x14, 0x80, 0x14, 0x00, 0xEE]);

        cpu.run();

        assert_eq!(cpu.registers[0], 45);
        assert_eq!(cpu.registers[ERRFLAG], 0);
    }
}