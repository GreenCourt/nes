use super::bus::{Bus, Mem};
use super::opcode::{AddressingMode, INSTRUCTIONS, Instruction, Mnemonic};

pub struct CPU {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    status: u8,
    stack_pointer: u8,
    program_counter: u16,
    pub bus: Bus,
}

impl Mem for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }

    fn mem_read_u16(&self, pos: u16) -> u16 {
        self.bus.mem_read_u16(pos)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        self.bus.mem_write_u16(pos, data)
    }
}

impl CPU {
    pub fn new(bus: Bus) -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            stack_pointer: 0,
            program_counter: 0,
            bus: bus,
        }
    }

    const STATUS_CARRY: u8 = 0b0000_0001;
    const STATUS_ZERO: u8 = 0b0000_0010;
    const STATUS_INTERRUPT_DISABLE: u8 = 0b0000_0100;
    const STATUS_DECIMAL_MODE: u8 = 0b0000_1000;
    const STATUS_BREAK: u8 = 0b0001_0000;
    const STATUS_RESERVED: u8 = 0b0010_0000;
    const STATUS_OVERFLOW: u8 = 0b0100_0000;
    const STATUS_NEGATIVE: u8 = 0b1000_0000;

    fn set_carry_flag(&mut self, flag: bool) {
        if flag {
            self.status |= CPU::STATUS_CARRY;
        } else {
            self.status &= !CPU::STATUS_CARRY;
        };
    }

    fn get_carry_flag(&mut self) -> bool {
        (self.status & CPU::STATUS_CARRY) != 0
    }

    fn update_zero_flag(&mut self, result: u8) {
        if result == 0 {
            self.status = self.status | CPU::STATUS_ZERO;
        } else {
            self.status = self.status & !CPU::STATUS_ZERO;
        }
    }

    pub fn get_zero_flag(&self) -> bool {
        (self.status & CPU::STATUS_ZERO) != 0
    }

    fn set_interrupt_disable_flag(&mut self, flag: bool) {
        if flag {
            self.status |= CPU::STATUS_INTERRUPT_DISABLE;
        } else {
            self.status &= !CPU::STATUS_INTERRUPT_DISABLE;
        };
    }

    fn set_decimal_mode_flag(&mut self, flag: bool) {
        if flag {
            self.status |= CPU::STATUS_DECIMAL_MODE;
        } else {
            self.status &= !CPU::STATUS_DECIMAL_MODE;
        };
    }

    fn set_break_flag(&mut self, flag: bool) {
        if flag {
            self.status |= CPU::STATUS_BREAK;
        } else {
            self.status &= !CPU::STATUS_BREAK;
        };
    }

    fn get_break_flag(&mut self) -> bool {
        (self.status & CPU::STATUS_BREAK) != 0
    }

    fn set_overflow_flag(&mut self, flag: bool) {
        if flag {
            self.status |= CPU::STATUS_OVERFLOW;
        } else {
            self.status &= !CPU::STATUS_OVERFLOW;
        };
    }

    fn get_overflow_flag(&mut self) -> bool {
        (self.status & CPU::STATUS_OVERFLOW) != 0
    }

    fn update_negative_flag(&mut self, result: u8) {
        if result & 0b1000_0000 != 0 {
            self.status = self.status | CPU::STATUS_NEGATIVE;
        } else {
            self.status = self.status & !CPU::STATUS_NEGATIVE;
        }
    }

    pub fn get_negative_flag(&self) -> bool {
        (self.status & CPU::STATUS_NEGATIVE) != 0
    }

    pub fn stack_push(&mut self, data: u8) {
        self.mem_write(0x0100 + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    pub fn stack_push_u16(&mut self, data: u16) {
        self.stack_push((data >> 8) as u8);
        self.stack_push((data & 0xFF) as u8);
    }

    pub fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read(0x0100 + self.stack_pointer as u16)
    }

    pub fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;
        hi << 8 | lo
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;
        self.stack_pointer = 0xFD;
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn reset_and_run(&mut self) {
        self.reset();
        self.run()
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter + 1,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter + 1) as u16,

            AddressingMode::ZeroPageX => {
                let pos = self.mem_read(self.program_counter + 1);
                // wrap as u8, then cast to u16
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }

            AddressingMode::ZeroPageY => {
                let pos = self.mem_read(self.program_counter + 1);
                // wrap as u8, then cast to u16
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter + 1),

            AddressingMode::AbsoluteX => {
                let base = self.mem_read_u16(self.program_counter + 1);
                // wrap as u16
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }

            AddressingMode::AbsoluteY => {
                let base = self.mem_read_u16(self.program_counter + 1);
                // wrap as u16
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::Indirect => {
                let ptr = self.mem_read_u16(self.program_counter + 1);
                let addr = if ptr & 0x00FF == 0x00FF {
                    // https://forums.nesdev.org/viewtopic.php?t=19140
                    let lo = self.mem_read(ptr) as u16;
                    let hi = self.mem_read(ptr & 0xFF00) as u16;
                    (hi << 8) | lo
                } else {
                    self.mem_read_u16(ptr)
                };
                addr
            }

            AddressingMode::IndirectX => {
                let ptr_base: u8 = self.mem_read(self.program_counter + 1);
                let ptr: u8 = ptr_base.wrapping_add(self.register_x); // wrap as u8

                let lo = self.mem_read(ptr as u16) as u16;
                let hi = self.mem_read(
                    ptr.wrapping_add(1) as u16, /* wrap as u8 then cast to u16 */
                ) as u16;
                (hi << 8) | lo // this is different to mem_read_u16 because of wrapping
            }

            AddressingMode::IndirectY => {
                let ptr: u8 = self.mem_read(self.program_counter + 1);

                let lo = self.mem_read(ptr as u16) as u16;
                let hi = self.mem_read(
                    ptr.wrapping_add(1) as u16, /* wrap as u8 then cast to u16 */
                ) as u16;

                let base = (hi << 8) | lo; // this is different to mem_read_u16 because of wrapping
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::Relative => {
                let offset: i8 = self.mem_read(self.program_counter + 1) as i8;
                self.program_counter
                    .wrapping_add(2)
                    .wrapping_add(offset as u16)
            }

            AddressingMode::Accumulator => {
                panic!("cannot resolve address for the Implied mode");
            }

            AddressingMode::Implied => {
                panic!("cannot resolve address for the Implied mode");
            }
        }
    }

    fn adc(&mut self, mode: &AddressingMode) {
        // Add with Carry
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let (sum1, carry1) = self.register_a.overflowing_add(value);
        let (result, carry2) = sum1.overflowing_add(self.get_carry_flag() as u8);
        self.set_carry_flag(carry1 || carry2);
        self.update_zero_flag(result);
        self.update_negative_flag(result);

        // set overflow flag if
        // (sign(register_a) != sign(result))
        // and (sign(value) != sign(result))
        self.set_overflow_flag(((self.register_a ^ result) & (value ^ result) & 0x80) != 0);

        // set register_a after all
        self.register_a = result;
    }

    fn and(&mut self, mode: &AddressingMode) {
        // Logical AND
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a &= value;
        self.update_zero_flag(self.register_a);
        self.update_negative_flag(self.register_a);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        // Arithmetic Shift Left
        let (result, carry) = if *mode == AddressingMode::Accumulator {
            let (result, carry) = self.register_a.overflowing_mul(2);
            self.register_a = result;
            (result, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let (result, carry) = value.overflowing_mul(2);
            self.mem_write(addr, result);
            (result, carry)
        };

        self.set_carry_flag(carry);
        self.update_zero_flag(result);
        self.update_negative_flag(result);
    }

    fn bcc(&mut self) {
        // Branch if Carry Clear
        if !self.get_carry_flag() {
            let addr = self.get_operand_address(&AddressingMode::Relative);
            self.program_counter = addr;
        }
    }

    fn bcs(&mut self) {
        // Branch if Carry Set
        if self.get_carry_flag() {
            let addr = self.get_operand_address(&AddressingMode::Relative);
            self.program_counter = addr;
        }
    }

    fn beq(&mut self) {
        // Branch if Equal
        if self.get_zero_flag() {
            let addr = self.get_operand_address(&AddressingMode::Relative);
            self.program_counter = addr;
        }
    }

    fn bit(&mut self, mode: &AddressingMode) {
        // Bit Test
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let result = value & self.register_a;
        self.update_zero_flag(result);
        self.set_overflow_flag(value & 0b0100_0000 != 0);
        self.update_negative_flag(value);
    }

    fn bmi(&mut self) {
        // Branch if Minus
        if self.get_negative_flag() {
            let addr = self.get_operand_address(&AddressingMode::Relative);
            self.program_counter = addr;
        }
    }

    fn bne(&mut self) {
        // Branch if Not Equal
        if !self.get_zero_flag() {
            let addr = self.get_operand_address(&AddressingMode::Relative);
            self.program_counter = addr;
        }
    }

    fn bpl(&mut self) {
        // Branch if Positive
        if !self.get_negative_flag() {
            let addr = self.get_operand_address(&AddressingMode::Relative);
            self.program_counter = addr;
        }
    }

    fn brk(&mut self) {
        //  Force Interrupt
        self.stack_push_u16(self.program_counter);
        self.stack_push(self.status);
        // TODO the IRQ interrupt vector at $FFFE/F is loaded into the PC
        self.set_break_flag(true);
    }

    fn bvc(&mut self) {
        // Branch if Overflow Clear
        if !self.get_overflow_flag() {
            let addr = self.get_operand_address(&AddressingMode::Relative);
            self.program_counter = addr;
        }
    }

    fn bvs(&mut self) {
        //Branch if Overflow Set
        if self.get_overflow_flag() {
            let addr = self.get_operand_address(&AddressingMode::Relative);
            self.program_counter = addr;
        }
    }

    fn clc(&mut self) {
        // Clear Carry Flag
        self.set_carry_flag(false);
    }

    fn cld(&mut self) {
        // Clear Decimal Mode
        self.set_decimal_mode_flag(false);
    }

    fn cli(&mut self) {
        // Clear Interrupt Disable
        self.set_interrupt_disable_flag(false);
    }

    fn clv(&mut self) {
        // Clear Overflow Flag
        self.set_overflow_flag(false);
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        // Compare
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_carry_flag(self.register_a >= value);
        self.update_zero_flag(self.register_a.wrapping_sub(value));
        self.update_negative_flag(self.register_a.wrapping_sub(value));
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        // Compare X Register
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_carry_flag(self.register_x >= value);
        self.update_zero_flag(self.register_x.wrapping_sub(value));
        self.update_negative_flag(self.register_x.wrapping_sub(value));
    }

    fn cpy(&mut self, mode: &AddressingMode) {
        // Compare Y Register
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.set_carry_flag(self.register_y >= value);
        self.update_zero_flag(self.register_y.wrapping_sub(value));
        self.update_negative_flag(self.register_y.wrapping_sub(value));
    }

    fn dec(&mut self, mode: &AddressingMode) {
        // Decrement Memory
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let result = value.wrapping_sub(1);
        self.mem_write(addr, result);
        self.update_zero_flag(result);
        self.update_negative_flag(result);
    }

    fn dex(&mut self) {
        // Decrement X Register
        let result = self.register_x.wrapping_sub(1);
        self.register_x = result;
        self.update_zero_flag(result);
        self.update_negative_flag(result);
    }

    fn dey(&mut self) {
        // Decrement Y Register
        let result = self.register_y.wrapping_sub(1);
        self.register_y = result;
        self.update_zero_flag(result);
        self.update_negative_flag(result);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        // Exclusive OR
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a ^= value;
        self.update_zero_flag(self.register_a);
        self.update_negative_flag(self.register_a);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        // Increment Memory
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let result = value.wrapping_add(1);
        self.mem_write(addr, result);
        self.update_zero_flag(result);
        self.update_negative_flag(result);
    }

    fn inx(&mut self) {
        // Increment X Register
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_flag(self.register_x);
        self.update_negative_flag(self.register_x);
    }

    fn iny(&mut self) {
        // Increment Y Register
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_flag(self.register_y);
        self.update_negative_flag(self.register_y);
    }

    fn jmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.program_counter = addr;
    }

    fn jsr(&mut self) {
        // Jump to Subroutine
        let addr = self.get_operand_address(&AddressingMode::Absolute);
        self.stack_push_u16(self.program_counter + 3 - 1); // add 3 because of Absolute
        self.program_counter = addr;
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_flag(self.register_a);
        self.update_negative_flag(self.register_a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_x = value;
        self.update_zero_flag(self.register_x);
        self.update_negative_flag(self.register_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_y = value;
        self.update_zero_flag(self.register_y);
        self.update_negative_flag(self.register_y);
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        // Logical Shift Right
        let (result, carry) = if *mode == AddressingMode::Accumulator {
            let carry = self.register_a & 0x1 != 0;
            self.register_a >>= 1;
            (self.register_a, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let carry = value & 0x1 != 0;
            let result = value >> 1;
            self.mem_write(addr, result);
            (result, carry)
        };

        self.set_carry_flag(carry);
        self.update_zero_flag(result);
        self.update_negative_flag(result);
    }

    fn ora(&mut self, mode: &AddressingMode) {
        // Logical Inclusive OR
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a |= value;
        self.update_zero_flag(self.register_a);
        self.update_negative_flag(self.register_a);
    }

    fn pha(&mut self) {
        // Push Accumulator
        self.stack_push(self.register_a);
    }

    fn php(&mut self) {
        // Push Processor Status
        self.stack_push(self.status | CPU::STATUS_BREAK | CPU::STATUS_RESERVED);
    }

    fn pla(&mut self) {
        // Pull Accumulator
        self.register_a = self.stack_pop();
        self.update_zero_flag(self.register_a);
        self.update_negative_flag(self.register_a);
    }

    fn plp(&mut self) {
        // Pull Processor Status
        self.status = self.stack_pop();
        self.status &= !CPU::STATUS_BREAK;
        self.status |= CPU::STATUS_RESERVED;
    }

    fn rol(&mut self, mode: &AddressingMode) {
        // Rotate Left
        let (result, carry) = if *mode == AddressingMode::Accumulator {
            let carry = (self.register_a & 0b1000_0000) != 0;
            let result =
                ((self.register_a & 0x7F) << 1) + if self.get_carry_flag() { 1 } else { 0 };
            self.register_a = result;
            (result, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);

            let carry = (value & 0b1000_0000) != 0;
            let result = ((value & 0x7F) << 1) + if self.get_carry_flag() { 1 } else { 0 };
            self.mem_write(addr, result);
            (result, carry)
        };

        self.set_carry_flag(carry);
        self.update_zero_flag(result);
        self.update_negative_flag(result);
    }

    fn ror(&mut self, mode: &AddressingMode) {
        // Rotate Right
        let (result, carry) = if *mode == AddressingMode::Accumulator {
            let carry = (self.register_a & 0b0000_0001) != 0;
            let result = ((self.register_a & 0xFE) >> 1)
                + if self.get_carry_flag() {
                    0b1000_0000
                } else {
                    0
                };
            self.register_a = result;
            (result, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);

            let carry = (value & 0b0000_0001) != 0;
            let result = ((value & 0xFE) >> 1)
                + if self.get_carry_flag() {
                    0b1000_0000
                } else {
                    0
                };
            self.mem_write(addr, result);
            (result, carry)
        };

        self.set_carry_flag(carry);
        self.update_zero_flag(result);
        self.update_negative_flag(result);
    }

    fn rti(&mut self) {
        // Return from Interrupt
        self.status = self.stack_pop();
        self.status &= !CPU::STATUS_BREAK;
        self.status |= CPU::STATUS_RESERVED;

        self.program_counter = self.stack_pop_u16();
        self.program_counter -= 1;
    }

    fn rts(&mut self) {
        // Return from Subroutine
        self.program_counter = self.stack_pop_u16() + 1;
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        // Subtract with Carry
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let value_to_add = (value as i8).wrapping_neg().wrapping_sub(1) as u8;

        let (sum1, carry1) = self.register_a.overflowing_add(value_to_add);
        let (result, carry2) = sum1.overflowing_add(self.get_carry_flag() as u8);
        self.set_carry_flag(carry1 || carry2);
        self.update_zero_flag(result);
        self.update_negative_flag(result);

        // set overflow flag if
        // (sign(register_a) != sign(result))
        // and (sign(value_to_add) != sign(result))
        self.set_overflow_flag(((self.register_a ^ result) & (value_to_add ^ result) & 0x80) != 0);

        // set register_a after all
        self.register_a = result;
    }

    fn sec(&mut self) {
        // Set Carry Flag
        self.set_carry_flag(true);
    }

    fn sed(&mut self) {
        // Set Decimal Flag
        self.set_decimal_mode_flag(true);
    }

    fn sei(&mut self) {
        // Set Interrupt Disable
        self.set_interrupt_disable_flag(true);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        // Store Accumulator
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        // Store X Register
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        // Store Y Register
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    fn tax(&mut self) {
        // Transfer Accumulator to X
        self.register_x = self.register_a;
        self.update_zero_flag(self.register_x);
        self.update_negative_flag(self.register_x);
    }

    fn tay(&mut self) {
        // Transfer Accumulator to Y
        self.register_y = self.register_a;
        self.update_zero_flag(self.register_y);
        self.update_negative_flag(self.register_y);
    }

    fn tsx(&mut self) {
        // Transfer Stack Pointer to X
        self.register_x = self.stack_pointer;
        self.update_zero_flag(self.register_x);
        self.update_negative_flag(self.register_x);
    }

    fn txa(&mut self) {
        // Transfer X to Accumulator
        self.register_a = self.register_x;
        self.update_zero_flag(self.register_a);
        self.update_negative_flag(self.register_a);
    }

    fn txs(&mut self) {
        // Transfer X to Stack Pointer
        self.stack_pointer = self.register_x;
    }

    fn tya(&mut self) {
        // Transfer Y to Accumulator
        self.register_a = self.register_y;
        self.update_zero_flag(self.register_a);
        self.update_negative_flag(self.register_a);
    }

    fn dcp(&mut self, mode: &AddressingMode) {
        // (unofficial) DEC then CMP
        self.dec(mode);
        self.cmp(mode);
    }

    fn isb(&mut self, mode: &AddressingMode) {
        // (unofficial) INC then SBC
        self.inc(mode);
        self.sbc(mode);
    }

    fn lax(&mut self, mode: &AddressingMode) {
        // (unofficial) LDA then TAX
        self.lda(mode);
        self.tax();
    }

    fn rla(&mut self, mode: &AddressingMode) {
        // (unofficial) ROL then AND
        self.rol(mode);
        self.and(mode);
    }

    fn rra(&mut self, mode: &AddressingMode) {
        // (unofficial) ROR then ADC
        self.ror(mode);
        self.adc(mode);
    }

    fn sax(&mut self, mode: &AddressingMode) {
        // (unofficial) Store Accumulator & X Register
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a & self.register_x);
    }

    fn slo(&mut self, mode: &AddressingMode) {
        // (unofficial) ASL the ORA
        self.asl(mode);
        self.ora(mode);
    }

    fn sre(&mut self, mode: &AddressingMode) {
        // (unofficial) LSR the EOR
        self.lsr(mode);
        self.eor(mode);
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        loop {
            callback(self);
            let opcode = self.mem_read(self.program_counter);
            let instruction: &Instruction = &INSTRUCTIONS[opcode as usize];
            let program_counter_before = self.program_counter;

            match instruction.mnemonic {
                Mnemonic::ADC => self.adc(&instruction.addressing_mode),
                Mnemonic::AND => self.and(&instruction.addressing_mode),
                Mnemonic::ASL => self.asl(&instruction.addressing_mode),
                Mnemonic::BCC => self.bcc(),
                Mnemonic::BCS => self.bcs(),
                Mnemonic::BEQ => self.beq(),
                Mnemonic::BIT => self.bit(&instruction.addressing_mode),
                Mnemonic::BMI => self.bmi(),
                Mnemonic::BNE => self.bne(),
                Mnemonic::BPL => self.bpl(),
                Mnemonic::BRK => self.brk(),
                Mnemonic::BVC => self.bvc(),
                Mnemonic::BVS => self.bvs(),
                Mnemonic::CLC => self.clc(),
                Mnemonic::CLD => self.cld(),
                Mnemonic::CLI => self.cli(),
                Mnemonic::CLV => self.clv(),
                Mnemonic::CMP => self.cmp(&instruction.addressing_mode),
                Mnemonic::CPX => self.cpx(&instruction.addressing_mode),
                Mnemonic::CPY => self.cpy(&instruction.addressing_mode),
                Mnemonic::DEC => self.dec(&instruction.addressing_mode),
                Mnemonic::DEX => self.dex(),
                Mnemonic::DEY => self.dey(),
                Mnemonic::EOR => self.eor(&instruction.addressing_mode),
                Mnemonic::INC => self.inc(&instruction.addressing_mode),
                Mnemonic::INX => self.inx(),
                Mnemonic::INY => self.iny(),
                Mnemonic::JMP => self.jmp(&instruction.addressing_mode),
                Mnemonic::JSR => self.jsr(),
                Mnemonic::LDA => self.lda(&instruction.addressing_mode),
                Mnemonic::LDX => self.ldx(&instruction.addressing_mode),
                Mnemonic::LDY => self.ldy(&instruction.addressing_mode),
                Mnemonic::LSR => self.lsr(&instruction.addressing_mode),
                Mnemonic::NOP => {}
                Mnemonic::ORA => self.ora(&instruction.addressing_mode),
                Mnemonic::PHA => self.pha(),
                Mnemonic::PHP => self.php(),
                Mnemonic::PLA => self.pla(),
                Mnemonic::PLP => self.plp(),
                Mnemonic::ROL => self.rol(&instruction.addressing_mode),
                Mnemonic::ROR => self.ror(&instruction.addressing_mode),
                Mnemonic::RTI => self.rti(),
                Mnemonic::RTS => self.rts(),
                Mnemonic::SBC => self.sbc(&instruction.addressing_mode),
                Mnemonic::SEC => self.sec(),
                Mnemonic::SED => self.sed(),
                Mnemonic::SEI => self.sei(),
                Mnemonic::STA => self.sta(&instruction.addressing_mode),
                Mnemonic::STX => self.stx(&instruction.addressing_mode),
                Mnemonic::STY => self.sty(&instruction.addressing_mode),
                Mnemonic::TAX => self.tax(),
                Mnemonic::TAY => self.tay(),
                Mnemonic::TSX => self.tsx(),
                Mnemonic::TXA => self.txa(),
                Mnemonic::TXS => self.txs(),
                Mnemonic::TYA => self.tya(),
                // --- unofficial ---
                Mnemonic::DCP => self.dcp(&instruction.addressing_mode),
                Mnemonic::ISB => self.isb(&instruction.addressing_mode),
                Mnemonic::LAX => self.lax(&instruction.addressing_mode),
                Mnemonic::RLA => self.rla(&instruction.addressing_mode),
                Mnemonic::RRA => self.rra(&instruction.addressing_mode),
                Mnemonic::SAX => self.sax(&instruction.addressing_mode),
                Mnemonic::SLO => self.slo(&instruction.addressing_mode),
                Mnemonic::SRE => self.sre(&instruction.addressing_mode),
                _ => panic!("unknown opcode: 0x{:x}", opcode),
            }

            if self.program_counter == program_counter_before {
                self.program_counter += match instruction.addressing_mode {
                    AddressingMode::Accumulator => 1,
                    AddressingMode::Immediate => 2,
                    AddressingMode::ZeroPage => 2,
                    AddressingMode::ZeroPageX => 2,
                    AddressingMode::ZeroPageY => 2,
                    AddressingMode::Absolute => 3,
                    AddressingMode::AbsoluteX => 3,
                    AddressingMode::AbsoluteY => 3,
                    AddressingMode::Indirect => 3,
                    AddressingMode::IndirectX => 2,
                    AddressingMode::IndirectY => 2,
                    AddressingMode::Relative => 2,
                    AddressingMode::Implied => 1,
                };
            }

            if self.get_break_flag() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cartridge::Cartridge;
    use std::fs;

    impl CPU {
        fn get_interrupt_disable_flag(&mut self) -> bool {
            (self.status & CPU::STATUS_INTERRUPT_DISABLE) != 0
        }

        fn get_decimal_mode_flag(&mut self) -> bool {
            (self.status & CPU::STATUS_DECIMAL_MODE) != 0
        }

        fn trace(&mut self) -> String {
            let opcode = self.mem_read(self.program_counter);
            let instruction: &Instruction = &INSTRUCTIONS[opcode as usize];
            let operand = match instruction.addressing_mode {
                AddressingMode::Immediate
                | AddressingMode::ZeroPage
                | AddressingMode::ZeroPageX
                | AddressingMode::ZeroPageY
                | AddressingMode::IndirectX
                | AddressingMode::IndirectY
                | AddressingMode::Relative => {
                    format!("{:02X}   ", self.mem_read(self.program_counter + 1))
                }

                AddressingMode::Absolute
                | AddressingMode::AbsoluteX
                | AddressingMode::AbsoluteY
                | AddressingMode::Indirect => format!(
                    "{:02X} {:02X}",
                    self.mem_read(self.program_counter + 1),
                    self.mem_read(self.program_counter + 2)
                ),

                AddressingMode::Accumulator | AddressingMode::Implied => String::from("     "),
            };

            let mnemonic: String = match instruction.mnemonic {
                Mnemonic::DCP => String::from("*DCP"),
                Mnemonic::ISB => String::from("*ISB"),
                Mnemonic::LAX => String::from("*LAX"),
                Mnemonic::NOP => String::from(if opcode == 0xEA { "NOP" } else { "*NOP" }),
                Mnemonic::RLA => String::from("*RLA"),
                Mnemonic::RRA => String::from("*RRA"),
                Mnemonic::SAX => String::from("*SAX"),
                Mnemonic::SBC => String::from(if opcode == 0xEB { "*SBC" } else { "SBC" }),
                Mnemonic::SLO => String::from("*SLO"),
                Mnemonic::SRE => String::from("*SRE"),
                _ => format!("{:?}", instruction.mnemonic),
            };

            let memory_value =
                if instruction.mnemonic == Mnemonic::JMP || instruction.mnemonic == Mnemonic::JSR {
                    match instruction.addressing_mode {
                        AddressingMode::Absolute => String::from(""),
                        AddressingMode::Indirect => {
                            let addr = self.get_operand_address(&instruction.addressing_mode);
                            format!(" = {:04X}", addr)
                        }
                        _ => {
                            panic!("invalid opcode");
                        }
                    }
                } else {
                    match instruction.addressing_mode {
                        AddressingMode::Immediate
                        | AddressingMode::Accumulator
                        | AddressingMode::Implied
                        | AddressingMode::Relative => String::from(""),
                        _ => {
                            let addr = self.get_operand_address(&instruction.addressing_mode);
                            format!(" = {:02X}", self.mem_read(addr))
                        }
                    }
                };

            let middle = match instruction.addressing_mode {
                AddressingMode::Immediate => {
                    format!("#${:02X}", self.mem_read(self.program_counter + 1))
                }
                AddressingMode::ZeroPage => {
                    format!("${:02X}", self.mem_read(self.program_counter + 1))
                }
                AddressingMode::ZeroPageX => format!(
                    "${:02X},X @ {:02X}",
                    self.mem_read(self.program_counter + 1),
                    self.get_operand_address(&AddressingMode::ZeroPageX),
                ),
                AddressingMode::ZeroPageY => format!(
                    "${:02X},Y @ {:02X}",
                    self.mem_read(self.program_counter + 1),
                    self.get_operand_address(&AddressingMode::ZeroPageY),
                ),
                AddressingMode::IndirectX => format!(
                    "(${:02X},X) @ {:02X} = {:04X}",
                    self.mem_read(self.program_counter + 1),
                    self.mem_read(self.program_counter + 1)
                        .wrapping_add(self.register_x) as u16,
                    self.get_operand_address(&AddressingMode::IndirectX),
                ),
                AddressingMode::IndirectY => format!(
                    "(${:02X}),Y = {:04X} @ {:04X}",
                    self.mem_read(self.program_counter + 1),
                    self.get_operand_address(&AddressingMode::IndirectY)
                        .wrapping_sub(self.register_y as u16),
                    self.get_operand_address(&AddressingMode::IndirectY),
                ),
                AddressingMode::Relative => format!(
                    "${:04X}",
                    self.get_operand_address(&AddressingMode::Relative)
                ),
                AddressingMode::Absolute => {
                    format!("${:04X}", self.mem_read_u16(self.program_counter + 1))
                }
                AddressingMode::AbsoluteX => format!(
                    "${:04X},X @ {:04X}",
                    self.mem_read_u16(self.program_counter + 1),
                    self.get_operand_address(&AddressingMode::AbsoluteX),
                ),
                AddressingMode::AbsoluteY => format!(
                    "${:04X},Y @ {:04X}",
                    self.mem_read_u16(self.program_counter + 1),
                    self.get_operand_address(&AddressingMode::AbsoluteY),
                ),
                AddressingMode::Indirect => {
                    format!("(${:04X})", self.mem_read_u16(self.program_counter + 1))
                }
                AddressingMode::Accumulator => String::from("A"),
                AddressingMode::Implied => String::from(""),
            } + &memory_value;

            format!(
                "{:04X}  {:02X} {} {:>4} {:27} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
                self.program_counter,
                opcode,
                operand,
                mnemonic,
                middle,
                self.register_a,
                self.register_x,
                self.register_y,
                self.status,
                self.stack_pointer
            )
        }
    }

    #[derive(Debug, PartialEq)]
    #[allow(non_camel_case_types)]
    pub enum Opcode {
        ADC_Immediate = 0x69,
        ADC_ZeroPage = 0x65,
        ADC_ZeroPageX = 0x75,
        ADC_Absolute = 0x6D,
        ADC_AbsoluteX = 0x7D,
        ADC_AbsoluteY = 0x79,
        ADC_IndirectX = 0x61,
        ADC_IndirectY = 0x71,
        AND_Immediate = 0x29,
        AND_ZeroPage = 0x25,
        AND_ZeroPageX = 0x35,
        AND_Absolute = 0x2D,
        AND_AbsoluteX = 0x3D,
        AND_AbsoluteY = 0x39,
        AND_IndirectX = 0x21,
        AND_IndirectY = 0x31,
        ASL_ZeroPage = 0x06,
        ASL_ZeroPageX = 0x16,
        ASL_Absolute = 0x0E,
        ASL_AbsoluteX = 0x1E,
        ASL_Accumulator = 0x0A,
        BCC_Relative = 0x90,
        BCS_Relative = 0xB0,
        BEQ_Relative = 0xF0,
        BIT_ZeroPage = 0x24,
        BIT_Absolute = 0x2C,
        BMI_Relative = 0x30,
        BNE_Relative = 0xD0,
        BPL_Relative = 0x10,
        BRK_Implied = 0x00,
        BVC_Relative = 0x50,
        BVS_Relative = 0x70,
        CLC_Implied = 0x18,
        CLD_Implied = 0xD8,
        CLI_Implied = 0x58,
        CLV_Implied = 0xB8,
        CMP_Immediate = 0xC9,
        CMP_ZeroPage = 0xC5,
        CMP_ZeroPageX = 0xD5,
        CMP_Absolute = 0xCD,
        CMP_AbsoluteX = 0xDD,
        CMP_AbsoluteY = 0xD9,
        CMP_IndirectX = 0xC1,
        CMP_IndirectY = 0xD1,
        CPX_Immediate = 0xE0,
        CPX_ZeroPage = 0xE4,
        CPX_Absolute = 0xEC,
        CPY_Immediate = 0xC0,
        CPY_ZeroPage = 0xC4,
        CPY_Absolute = 0xCC,
        DEC_ZeroPage = 0xC6,
        DEC_ZeroPageX = 0xD6,
        DEC_Absolute = 0xCE,
        DEC_AbsoluteX = 0xDE,
        DEX_Implied = 0xCA,
        DEY_Implied = 0x88,
        EOR_Immediate = 0x49,
        EOR_ZeroPage = 0x45,
        EOR_ZeroPageX = 0x55,
        EOR_Absolute = 0x4D,
        EOR_AbsoluteX = 0x5D,
        EOR_AbsoluteY = 0x59,
        EOR_IndirectX = 0x41,
        EOR_IndirectY = 0x51,
        INC_ZeroPage = 0xE6,
        INC_ZeroPageX = 0xF6,
        INC_Absolute = 0xEE,
        INC_AbsoluteX = 0xFE,
        INX_Implied = 0xE8,
        INY_Implied = 0xC8,
        JMP_Absolute = 0x4C,
        JMP_Indirect = 0x6C,
        JSR_Absolute = 0x20,
        LDA_Immediate = 0xA9,
        LDA_ZeroPage = 0xA5,
        LDA_ZeroPageX = 0xB5,
        LDA_Absolute = 0xAD,
        LDA_AbsoluteX = 0xBD,
        LDA_AbsoluteY = 0xB9,
        LDA_IndirectX = 0xA1,
        LDA_IndirectY = 0xB1,
        LDX_Immediate = 0xA2,
        LDX_ZeroPage = 0xA6,
        LDX_ZeroPageY = 0xB6,
        LDX_Absolute = 0xAE,
        LDX_AbsoluteY = 0xBE,
        LDY_Immediate = 0xA0,
        LDY_ZeroPage = 0xA4,
        LDY_ZeroPageX = 0xB4,
        LDY_Absolute = 0xAC,
        LDY_AbsoluteX = 0xBC,
        LSR_ZeroPage = 0x46,
        LSR_ZeroPageX = 0x56,
        LSR_Absolute = 0x4E,
        LSR_AbsoluteX = 0x5E,
        LSR_Accumulator = 0x4A,
        NOP_Implied = 0xEA,
        ORA_Immediate = 0x09,
        ORA_ZeroPage = 0x05,
        ORA_ZeroPageX = 0x15,
        ORA_Absolute = 0x0D,
        ORA_AbsoluteX = 0x1D,
        ORA_AbsoluteY = 0x19,
        ORA_IndirectX = 0x01,
        ORA_IndirectY = 0x11,
        PHA_Implied = 0x48,
        PHP_Implied = 0x08,
        PLA_Implied = 0x68,
        PLP_Implied = 0x28,
        ROL_ZeroPage = 0x26,
        ROL_ZeroPageX = 0x36,
        ROL_Absolute = 0x2E,
        ROL_AbsoluteX = 0x3E,
        ROL_Accumulator = 0x2A,
        ROR_ZeroPage = 0x66,
        ROR_ZeroPageX = 0x76,
        ROR_Absolute = 0x6E,
        ROR_AbsoluteX = 0x7E,
        ROR_Accumulator = 0x6A,
        RTI_Implied = 0x40,
        RTS_Implied = 0x60,
        SBC_Immediate = 0xE9,
        SBC_ZeroPage = 0xE5,
        SBC_ZeroPageX = 0xF5,
        SBC_Absolute = 0xED,
        SBC_AbsoluteX = 0xFD,
        SBC_AbsoluteY = 0xF9,
        SBC_IndirectX = 0xE1,
        SBC_IndirectY = 0xF1,
        SEC_Implied = 0x38,
        SED_Implied = 0xF8,
        SEI_Implied = 0x78,
        STA_ZeroPage = 0x85,
        STA_ZeroPageX = 0x95,
        STA_Absolute = 0x8D,
        STA_AbsoluteX = 0x9D,
        STA_AbsoluteY = 0x99,
        STA_IndirectX = 0x81,
        STA_IndirectY = 0x91,
        STX_ZeroPage = 0x86,
        STX_ZeroPageY = 0x96,
        STX_Absolute = 0x8E,
        STY_ZeroPage = 0x84,
        STY_ZeroPageX = 0x94,
        STY_Absolute = 0x8C,
        TAX_Implied = 0xAA,
        TAY_Implied = 0xA8,
        TSX_Implied = 0xBA,
        TXA_Implied = 0x8A,
        TXS_Implied = 0x9A,
        TYA_Implied = 0x98,
    }

    #[test]
    fn test_stack() {
        let ops = vec![Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.stack_push(0x11);
        cpu.stack_push_u16(0x2222);
        cpu.stack_push(0x33);
        cpu.stack_push_u16(0x4444);
        assert_eq!(cpu.stack_pop_u16(), 0x4444);
        assert_eq!(cpu.stack_pop(), 0x33);
        assert_eq!(cpu.stack_pop_u16(), 0x2222);
        assert_eq!(cpu.stack_pop(), 0x11);
    }

    #[test]
    fn test_adc_immediate() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x1A,
            Opcode::ADC_Immediate as u8,
            0x21,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x3B);
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_adc_zeroflag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xFF,
            Opcode::ADC_Immediate as u8,
            0x01,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_overflow_flag());
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_adc_zeropage() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xA2,
            Opcode::ADC_ZeroPage as u8,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x12, 0xB3);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x55);
        assert!(cpu.get_carry_flag());
        assert!(cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_adc_zeropage_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xA2,
            Opcode::LDX_Immediate as u8,
            0x04,
            Opcode::ADC_ZeroPageX as u8,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x16, 0xE3);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x85);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_adc_absolute() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x10,
            Opcode::ADC_Absolute as u8,
            0xAB,
            0x16,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x16AB, 0x72);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x82);
        assert!(!cpu.get_carry_flag());
        assert!(cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_adc_absolute_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x50,
            Opcode::LDX_Immediate as u8,
            0x04,
            Opcode::ADC_AbsoluteX as u8,
            0xA7,
            0x16,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x16AB, 0x50);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0xA0);
        assert!(!cpu.get_carry_flag());
        assert!(cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_adc_absolute_y() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x11,
            Opcode::LDY_Immediate as u8,
            0x04,
            Opcode::ADC_AbsoluteY as u8,
            0xB7,
            0x17,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x17BB, 0x74);
        cpu.reset_and_run();

        assert_eq!(cpu.register_a, 0x85);
        assert!(!cpu.get_carry_flag());
        assert!(cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_adc_indirect_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xA2,
            Opcode::LDX_Immediate as u8,
            0x04,
            Opcode::ADC_IndirectX as u8,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x16, 0x1E2B);
        cpu.mem_write(0x1E2B, 0xD8);
        cpu.reset_and_run();

        assert_eq!(cpu.register_a, 0x7A);
        assert!(cpu.get_carry_flag());
        assert!(cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_adc_indirect_y() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xA2,
            Opcode::LDY_Immediate as u8,
            0x04,
            Opcode::ADC_IndirectY as u8,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x12, 0x1B2B);
        cpu.mem_write(0x1B2F, 0xD8);
        cpu.reset_and_run();

        assert_eq!(cpu.register_a, 0x7A);
        assert!(cpu.get_carry_flag());
        assert!(cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_and_immediate() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_1101,
            Opcode::AND_Immediate as u8,
            0b1011_1001,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b1001_1001);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_and_zeroflag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0100_0001,
            Opcode::AND_Immediate as u8,
            0b1011_1100,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0000_0000);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_and_zeropage() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0100_1101,
            Opcode::AND_ZeroPage as u8,
            0x87,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x87, 0b1011_1100);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0000_1100);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_and_zeropage_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0100_1101,
            Opcode::LDX_Immediate as u8,
            4,
            Opcode::AND_ZeroPageX as u8,
            0x83,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x87, 0b1011_1100);
        cpu.reset_and_run();

        assert_eq!(cpu.register_a, 0b0000_1100);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_and_absolute() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0110_1101,
            Opcode::AND_Absolute as u8,
            0xE1,
            0x1A,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x1AE1, 0b1011_1100);
        cpu.reset_and_run();

        assert_eq!(cpu.register_a, 0b0010_1100);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_and_absolute_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1110_1101,
            Opcode::LDX_Immediate as u8,
            0x3,
            Opcode::AND_AbsoluteX as u8,
            0xDE,
            0x18,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x18E1, 0b1011_1100);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b1010_1100);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_and_absolute_y() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0111_1101,
            Opcode::LDY_Immediate as u8,
            0x6,
            Opcode::AND_AbsoluteY as u8,
            0xDB,
            0x0E,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x0EE1, 0b1011_1100);
        cpu.reset_and_run();

        assert_eq!(cpu.register_a, 0b0011_1100);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_and_indirect_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0111_1101,
            Opcode::LDX_Immediate as u8,
            0x5,
            Opcode::AND_IndirectX as u8,
            0xDC,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0xE1, 0x1CF7);
        cpu.mem_write(0x1CF7, 0b1011_1100);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0011_1100);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_and_indirect_y() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0111_1101,
            Opcode::LDY_Immediate as u8,
            0x5,
            Opcode::AND_IndirectY as u8,
            0xE1,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0xE1, 0x0CF7);
        cpu.mem_write(0x0CFC, 0b1011_1100);
        cpu.reset_and_run();

        assert_eq!(cpu.register_a, 0b0011_1100);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_asl_accumulator() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0111_1101,
            Opcode::ASL_Accumulator as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();

        assert_eq!(cpu.register_a, 0b1111_1010);
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_asl_zeroflag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1000_0000,
            Opcode::ASL_Accumulator as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0000_0000);
        assert!(cpu.get_carry_flag());
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_asl_zeropage() {
        let ops = vec![Opcode::ASL_ZeroPage as u8, 0x5A, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x5A, 0b0010_0000);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x5A), 0b0100_0000);
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_asl_zeropage_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x03,
            Opcode::ASL_ZeroPageX as u8,
            0x5A,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x5D, 0b1001_0010);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x5D), 0b0010_0100);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_asl_absolute() {
        let ops = vec![
            Opcode::ASL_Absolute as u8,
            0x5D,
            0x1B,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x1B5D, 0b0101_0010);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x1B5D), 0b1010_0100);
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_asl_absolute_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x04,
            Opcode::ASL_AbsoluteX as u8,
            0x59,
            0x1B,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x1B5D, 0b0001_0110);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x1B5D), 0b0010_1100);
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_bcc_relative() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0000_0100,
            Opcode::ASL_Accumulator as u8,
            Opcode::BCC_Relative as u8,
            0x0A,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8010);
    }

    #[test]
    fn test_bcc_no_jump() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1000_0000,
            Opcode::ASL_Accumulator as u8,
            Opcode::BCC_Relative as u8,
            0xAA,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8006);
    }

    #[test]
    fn test_bcs_relative() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1000_0000,
            Opcode::ASL_Accumulator as u8,
            Opcode::BCS_Relative as u8,
            0x0B,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8011);
    }

    #[test]
    fn test_bcs_no_jump() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0000_0100,
            Opcode::ASL_Accumulator as u8,
            Opcode::BCS_Relative as u8,
            0xAA,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8006);
    }

    #[test]
    fn test_beq_relative() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0000_0000,
            Opcode::ASL_Accumulator as u8,
            Opcode::BEQ_Relative as u8,
            0x0C,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8012);
    }

    #[test]
    fn test_beq_no_jump() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1101_0000,
            Opcode::ASL_Accumulator as u8,
            Opcode::BEQ_Relative as u8,
            0xAA,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8006);
    }

    #[test]
    fn test_bit_zeropage() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1101_0010,
            Opcode::BIT_ZeroPage as u8,
            0xAD,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0xAD, 0b0010_0011);
        cpu.reset_and_run();
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_overflow_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_bit_zero_flag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1101_0100,
            Opcode::BIT_ZeroPage as u8,
            0xAD,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0xAD, 0b0010_0011);
        cpu.reset_and_run();
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_overflow_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_bit_overflow_flag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_0101,
            Opcode::BIT_ZeroPage as u8,
            0xAD,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0xAD, 0b0110_0011);
        cpu.reset_and_run();
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_overflow_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_bit_negative_flag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0001_0101,
            Opcode::BIT_ZeroPage as u8,
            0xAD,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0xAD, 0b1010_0011);
        cpu.reset_and_run();
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_overflow_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_bit_absolute() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0001_0010,
            Opcode::BIT_Absolute as u8,
            0xBD,
            0x0D,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x0DBD, 0b1110_0011);
        cpu.reset_and_run();
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_overflow_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_bmi_relative() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0100_0000,
            Opcode::ASL_Accumulator as u8,
            Opcode::BMI_Relative as u8,
            0x0D,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8013);
    }

    #[test]
    fn test_bmi_no_jump() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_0000,
            Opcode::ASL_Accumulator as u8,
            Opcode::BMI_Relative as u8,
            0xAA,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8006);
    }

    #[test]
    fn test_bne_relative() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0001_0000,
            Opcode::ASL_Accumulator as u8,
            Opcode::BNE_Relative as u8,
            0x0E,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8014);
    }

    #[test]
    fn test_bne_no_jump() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1000_0000,
            Opcode::ASL_Accumulator as u8,
            Opcode::BNE_Relative as u8,
            0xAA,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8006);
    }

    #[test]
    fn test_bpl_relative() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1010_0000,
            Opcode::ASL_Accumulator as u8,
            Opcode::BPL_Relative as u8,
            0x0F,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8015);
    }

    #[test]
    fn test_bpl_no_jump() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1101_0000,
            Opcode::ASL_Accumulator as u8,
            Opcode::BPL_Relative as u8,
            0xAA,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8006);
    }

    #[test]
    fn test_brk_implied() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xF0,
            Opcode::NOP_Implied as u8,
            Opcode::NOP_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.stack_pop(), CPU::STATUS_NEGATIVE);
        assert_eq!(cpu.stack_pop_u16(), 0x8004);
        assert!(cpu.get_break_flag());
    }

    #[test]
    fn test_bvc_relative() {
        let ops = vec![
            Opcode::BIT_ZeroPage as u8,
            0xCC,
            Opcode::BVC_Relative as u8,
            0x09,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0xCC, 0b1000_0000);
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x800E);
    }

    #[test]
    fn test_bvc_no_jump() {
        let ops = vec![
            Opcode::BIT_ZeroPage as u8,
            0xCC,
            Opcode::BVC_Relative as u8,
            0xAA,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0xCC, 0b0100_0000);
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8005);
    }

    #[test]
    fn test_bvs_relative() {
        let ops = vec![
            Opcode::BIT_ZeroPage as u8,
            0xCC,
            Opcode::BVS_Relative as u8,
            0x08,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0xCC, 0b0100_0000);
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x800D);
    }

    #[test]
    fn test_bvs_no_jump() {
        let ops = vec![
            Opcode::BIT_ZeroPage as u8,
            0xCC,
            Opcode::BVS_Relative as u8,
            0xAA,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0xCC, 0b1000_0000);
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x8005);
    }

    #[test]
    fn test_clc_implied() {
        let ops = vec![
            Opcode::SEC_Implied as u8,
            Opcode::CLC_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert!(!cpu.get_carry_flag());
    }

    #[test]
    fn test_cld_implied() {
        let ops = vec![
            Opcode::SED_Implied as u8,
            Opcode::CLD_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert!(!cpu.get_decimal_mode_flag());
    }

    #[test]
    fn test_cli_implied() {
        let ops = vec![
            Opcode::SEI_Implied as u8,
            Opcode::CLI_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert!(!cpu.get_interrupt_disable_flag());
    }

    #[test]
    fn test_clv_implied() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_0101,
            Opcode::BIT_ZeroPage as u8,
            0xAD,
            Opcode::CLV_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0xAD, 0b0110_0011);
        cpu.reset_and_run();
        assert!(!cpu.get_overflow_flag());
    }

    #[test]
    fn test_cmp_immediate() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x71,
            Opcode::CMP_Immediate as u8,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_cmp_zeropage() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x71,
            Opcode::CMP_ZeroPage as u8,
            0x9D,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x9D, 0x71);
        cpu.reset_and_run();
        assert!(cpu.get_carry_flag());
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_cmp_zeropage_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x71,
            Opcode::LDX_Immediate as u8,
            0x03,
            Opcode::CMP_ZeroPageX as u8,
            0x9D,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0xA0, 0x81);
        cpu.reset_and_run();
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_cmp_absolute() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xA1,
            Opcode::CMP_Absolute as u8,
            0x12,
            0x1A,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x1A12, 0x11);
        cpu.reset_and_run();
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_cmp_absolute_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x05,
            Opcode::LDX_Immediate as u8,
            0x03,
            Opcode::CMP_AbsoluteX as u8,
            0x9D,
            0x19,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x19A0, 0x10);
        cpu.reset_and_run();
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_cmp_absolute_y() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x05,
            Opcode::LDY_Immediate as u8,
            0x03,
            Opcode::CMP_AbsoluteY as u8,
            0x9D,
            0x19,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x19A0, 0x10);
        cpu.reset_and_run();
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_cmp_indirect_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x05,
            Opcode::LDX_Immediate as u8,
            0x03,
            Opcode::CMP_IndirectX as u8,
            0x76,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x79, 0x12BD);
        cpu.mem_write(0x12BD, 0x10);
        cpu.reset_and_run();
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_cmp_indirect_y() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x05,
            Opcode::LDY_Immediate as u8,
            0x03,
            Opcode::CMP_IndirectY as u8,
            0x79,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x79, 0x12BD);
        cpu.mem_write(0x12C0, 0x10);
        cpu.reset_and_run();
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_cpx_immediate() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x71,
            Opcode::CPX_Immediate as u8,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_cpx_zeropage() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x71,
            Opcode::CPX_ZeroPage as u8,
            0x9D,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x9D, 0x71);
        cpu.reset_and_run();
        assert!(cpu.get_carry_flag());
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_cpx_absolute() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0xA1,
            Opcode::CPX_Absolute as u8,
            0x41,
            0x1A,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x1A41, 0x11);
        cpu.reset_and_run();
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_cpy_immediate() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x71,
            Opcode::CPY_Immediate as u8,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_cpy_zeropage() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x71,
            Opcode::CPY_ZeroPage as u8,
            0x9D,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x9D, 0x71);
        cpu.reset_and_run();
        assert!(cpu.get_carry_flag());
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_cpy_absolute() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0xA1,
            Opcode::CPY_Absolute as u8,
            0x41,
            0x1A,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x1A41, 0x11);
        cpu.reset_and_run();
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_dec_zeropage() {
        let ops = vec![Opcode::DEC_ZeroPage as u8, 0x41, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x41, 0x11);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x41), 0x10);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_dec_zeroflag() {
        let ops = vec![Opcode::DEC_ZeroPage as u8, 0x41, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x41, 0x01);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x41), 0x00);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_dec_zeropage_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x03,
            Opcode::DEC_ZeroPageX as u8,
            0x3e,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x41, 0x00);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x41), 0xFF);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_dec_absolute() {
        let ops = vec![
            Opcode::DEC_Absolute as u8,
            0xAE,
            0x07,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x07AE, 0xD7);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x07AE), 0xD6);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_dec_absolute_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x05,
            Opcode::DEC_AbsoluteX as u8,
            0xA9,
            0x1D,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x1DAE, 0xD7);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x1DAE), 0xD6);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_dex_implied() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x05,
            Opcode::DEX_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0x04);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_dex_zeroflag() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x01,
            Opcode::DEX_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0x00);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_dex_negative_flag() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x00,
            Opcode::DEX_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0xFF);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_dey_implied() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x05,
            Opcode::DEY_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0x04);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_dey_zeroflag() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x01,
            Opcode::DEY_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0x00);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_dey_negative_flag() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x00,
            Opcode::DEY_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0xFF);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_eor_immediate() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_0110,
            Opcode::EOR_Immediate as u8,
            0b1101_0011,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0100_0101);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_eor_zeroflag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_0110,
            Opcode::EOR_Immediate as u8,
            0b1001_0110,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0000_0000);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_eor_zeropage() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_0110,
            Opcode::EOR_ZeroPage as u8,
            0xA2,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0xA2, 0b0101_0011);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b1100_0101);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_eor_zeropage_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_0110,
            Opcode::LDX_Immediate as u8,
            0x04,
            Opcode::EOR_ZeroPageX as u8,
            0x9E,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0xA2, 0b1101_0011);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0100_0101);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_eor_absolute() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_0110,
            Opcode::EOR_Absolute as u8,
            0xA2,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x12A2, 0b0101_0011);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b1100_0101);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_eor_absolute_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_0110,
            Opcode::LDX_Immediate as u8,
            0x10,
            Opcode::EOR_AbsoluteX as u8,
            0x92,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x12A2, 0b1101_0011);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0100_0101);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_eor_absolute_y() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_0110,
            Opcode::LDY_Immediate as u8,
            0x10,
            Opcode::EOR_AbsoluteY as u8,
            0x92,
            0x1E,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x1EA2, 0b1101_0011);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0100_0101);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_eor_indirect_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_0110,
            Opcode::LDX_Immediate as u8,
            0x10,
            Opcode::EOR_IndirectX as u8,
            0x92,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0xA2, 0x15E7);
        cpu.mem_write(0x15E7, 0b1101_0011);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0100_0101);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_eor_indirect_y() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_0110,
            Opcode::LDY_Immediate as u8,
            0x05,
            Opcode::EOR_IndirectY as u8,
            0xA2,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0xA2, 0x15E2);
        cpu.mem_write(0x15E7, 0b1101_0011);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0100_0101);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_inc_zeropage() {
        let ops = vec![Opcode::INC_ZeroPage as u8, 0x41, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x41, 0x11);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x41), 0x12);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_inc_zeroflag() {
        let ops = vec![Opcode::INC_ZeroPage as u8, 0x41, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x41, 0xFF);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x41), 0x00);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_inc_zeropage_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x03,
            Opcode::INC_ZeroPageX as u8,
            0x3e,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x41, 0xF0);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x41), 0xF1);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_inc_absolute() {
        let ops = vec![
            Opcode::INC_Absolute as u8,
            0xAE,
            0x07,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x07AE, 0xD7);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x07AE), 0xD8);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_inc_absolute_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x05,
            Opcode::INC_AbsoluteX as u8,
            0xA9,
            0x0A,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x0AAE, 0xD7);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x0AAE), 0xD8);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_inx_implied() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x05,
            Opcode::INX_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0x06);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_inx_zeroflag() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0xFF,
            Opcode::INX_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0x00);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_inx_negative_flag() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0xFA,
            Opcode::INX_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0xFB);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_iny_implied() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x05,
            Opcode::INY_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0x06);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_iny_zeroflag() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0xFF,
            Opcode::INY_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0x00);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_iny_negative_flag() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0xF4,
            Opcode::INY_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0xF5);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_jmp_absolute() {
        let ops = vec![
            Opcode::JMP_Absolute as u8,
            0xAD,
            0x18,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x18AD, Opcode::BRK_Implied as u8);
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x18AE);
    }

    #[test]
    fn test_jmp_indirect() {
        let ops = vec![
            Opcode::JMP_Indirect as u8,
            0xB1,
            0x18,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x18B1, 0x80AD);
        cpu.reset_and_run();
        assert_eq!(cpu.program_counter, 0x80AE);
    }

    #[test]
    fn test_subroutine() {
        // test jsr and rts
        let ops = vec![
            Opcode::JSR_Absolute as u8,
            0x06,
            0x80,
            Opcode::LDA_Immediate as u8,
            0xF5,
            Opcode::BRK_Implied as u8,
            Opcode::LDX_Immediate as u8,
            0x12,
            Opcode::LDA_Immediate as u8,
            0x33,
            Opcode::RTS_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();

        assert_eq!(cpu.program_counter, 0x8006);
        assert_eq!(cpu.register_a, 0xF5);
        assert_eq!(cpu.register_x, 0x12);

        // pushed by BRK
        assert_eq!(cpu.stack_pop(), CPU::STATUS_NEGATIVE);
        assert_eq!(cpu.stack_pop_u16(), 0x8005);
    }

    #[test]
    fn test_lda_immediate() {
        let ops = vec![Opcode::LDA_Immediate as u8, 0x1A, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x1A);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_lda_zero_flag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x1A,
            Opcode::LDA_Immediate as u8,
            0x00,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_lda_zeropage() {
        let ops = vec![Opcode::LDA_ZeroPage as u8, 0x12, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x12, 0xF4);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0xF4);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_lda_zeropage_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x04,
            Opcode::LDA_ZeroPageX as u8,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x16, 0x56);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x56);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_lda_absolute() {
        let ops = vec![
            Opcode::LDA_Absolute as u8,
            0x25,
            0x14,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x1425, 0x61);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x61);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_lda_absolute_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x10,
            Opcode::LDA_AbsoluteX as u8,
            0x25,
            0x1B,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x1B35, 0x21);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x21);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_lda_absolute_y() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x20,
            Opcode::LDA_AbsoluteY as u8,
            0x25,
            0x0D,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x0D45, 0x28);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x28);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_lda_indirect_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x07,
            Opcode::LDA_IndirectX as u8,
            0x80,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x87, 0x34);
        cpu.mem_write_u16(0x88, 0x16);
        cpu.mem_write_u16(0x1634, 0x55);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x55);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_lda_indirect_y() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x04,
            Opcode::LDA_IndirectY as u8,
            0x83,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x83, 0x54);
        cpu.mem_write_u16(0x84, 0x16);
        cpu.mem_write_u16(0x1658, 0x57);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x57);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ldx_immediate() {
        let ops = vec![Opcode::LDX_Immediate as u8, 0x1A, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0x1A);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ldx_zero_flag() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x1A,
            Opcode::LDX_Immediate as u8,
            0x00,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0x00);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ldx_zeropage() {
        let ops = vec![Opcode::LDX_ZeroPage as u8, 0x12, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x12, 0xF4);
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0xF4);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_ldx_zeropage_y() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x06,
            Opcode::LDX_ZeroPageY as u8,
            0x13,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x19, 0x57);
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0x06);
        assert_eq!(cpu.register_x, 0x57);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ldx_absolute() {
        let ops = vec![
            Opcode::LDX_Absolute as u8,
            0x25,
            0x14,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x1425, 0x61);
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0x61);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ldx_absolute_y() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x20,
            Opcode::LDX_AbsoluteY as u8,
            0x25,
            0x14,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x1445, 0x28);
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0x28);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ldy_immediate() {
        let ops = vec![Opcode::LDY_Immediate as u8, 0x1A, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0x1A);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ldy_zero_flag() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x1A,
            Opcode::LDY_Immediate as u8,
            0x00,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0x00);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ldy_zeropage() {
        let ops = vec![Opcode::LDY_ZeroPage as u8, 0x12, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x12, 0xF4);
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0xF4);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_ldy_zeropage_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x04,
            Opcode::LDY_ZeroPageX as u8,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x16, 0x56);
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0x56);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ldy_absolute() {
        let ops = vec![
            Opcode::LDY_Absolute as u8,
            0x25,
            0x0E,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x0E25, 0x61);
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0x61);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ldy_absolute_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x10,
            Opcode::LDY_AbsoluteX as u8,
            0x25,
            0x0C,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x0C35, 0x21);
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0x21);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_lsr_accumulator() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0111_1101,
            Opcode::LSR_Accumulator as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0011_1110);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_lsr_zeroflag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0000_0001,
            Opcode::LSR_Accumulator as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0000_0000);
        assert!(cpu.get_carry_flag());
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_lsr_zeropage() {
        let ops = vec![Opcode::LSR_ZeroPage as u8, 0x7E, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x7E, 0b0000_1001);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x7E), 0b0000_0100);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_lsr_zeropage_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x03,
            Opcode::LSR_ZeroPageX as u8,
            0x5A,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x5D, 0b0110_0000);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x5D), 0b0011_0000);
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_lsr_absolute() {
        let ops = vec![
            Opcode::LSR_Absolute as u8,
            0x12,
            0x1E,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x1E12, 0b0110_1001);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x1E12), 0b0011_0100);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_lsr_absolute_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x04,
            Opcode::LSR_AbsoluteX as u8,
            0x0E,
            0x1E,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x1E12, 0b0110_1001);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x1E12), 0b0011_0100);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ora_immediate() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1001_1101,
            Opcode::ORA_Immediate as u8,
            0b1011_1001,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b1011_1101);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_ora_zeroflag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0000_0000,
            Opcode::ORA_Immediate as u8,
            0b0000_0000,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0000_0000);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ora_zeropage() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0100_1101,
            Opcode::ORA_ZeroPage as u8,
            0x87,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x87, 0b0011_1100);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0111_1101);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ora_zeropage_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0100_1101,
            Opcode::LDX_Immediate as u8,
            4,
            Opcode::ORA_ZeroPageX as u8,
            0x83,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x87, 0b0011_1100);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0111_1101);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ora_absolute() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0110_1101,
            Opcode::ORA_Absolute as u8,
            0xE1,
            0x17,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x17E1, 0b0011_1100);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0111_1101);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ora_absolute_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1110_1101,
            Opcode::LDX_Immediate as u8,
            0x3,
            Opcode::ORA_AbsoluteX as u8,
            0xDE,
            0x17,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x17E1, 0b1011_1100);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b1111_1101);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_ora_absolute_y() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0111_1101,
            Opcode::LDY_Immediate as u8,
            0x6,
            Opcode::ORA_AbsoluteY as u8,
            0xDB,
            0x17,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x17E1, 0b0011_1100);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0111_1101);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ora_indirect_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0111_1101,
            Opcode::LDX_Immediate as u8,
            0x5,
            Opcode::ORA_IndirectX as u8,
            0xDC,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0xE1, 0x1CF7);
        cpu.mem_write(0x1CF7, 0b1011_1100);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b1111_1101);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_ora_indirect_y() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0111_1101,
            Opcode::LDY_Immediate as u8,
            0x5,
            Opcode::ORA_IndirectY as u8,
            0xE1,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0xE1, 0x1CF7);
        cpu.mem_write(0x1CFC, 0b1011_1100);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b1111_1101);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_pha_implied() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xFA,
            Opcode::PHA_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();

        // pushed by BRK
        assert_eq!(cpu.stack_pop(), CPU::STATUS_NEGATIVE);
        assert_eq!(cpu.stack_pop_u16(), 0x8003);

        // pushed by PHA
        assert_eq!(cpu.stack_pop(), 0xFA);
    }

    #[test]
    fn test_php_implied() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xFA,
            Opcode::PHP_Implied as u8,
            Opcode::LDA_Immediate as u8,
            0x02,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();

        // pushed by BRK
        assert_eq!(cpu.stack_pop(), 0x00);
        assert_eq!(cpu.stack_pop_u16(), 0x8005);

        // pushed by PHP
        assert_eq!(
            cpu.stack_pop(),
            CPU::STATUS_NEGATIVE | CPU::STATUS_BREAK | CPU::STATUS_RESERVED
        );
    }

    #[test]
    fn test_pla_implied() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xFA,
            Opcode::PHA_Implied as u8,
            Opcode::LDA_Immediate as u8,
            0x02,
            Opcode::PLA_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0xFA);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_pla_zeroflag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x00,
            Opcode::PHA_Implied as u8,
            Opcode::LDA_Immediate as u8,
            0x02,
            Opcode::PLA_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_plp_implied() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xFF,
            Opcode::PHP_Implied as u8,
            Opcode::LDA_Immediate as u8,
            0x02,
            Opcode::PLP_Implied as u8,
            Opcode::PHP_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();

        // pushed by BRK
        assert_eq!(cpu.stack_pop(), CPU::STATUS_NEGATIVE | CPU::STATUS_RESERVED);
        assert_eq!(cpu.stack_pop_u16(), 0x8007);

        // pushed by PHP
        assert_eq!(
            cpu.stack_pop(),
            CPU::STATUS_NEGATIVE | CPU::STATUS_BREAK | CPU::STATUS_RESERVED
        );
    }

    #[test]
    fn test_rol_accumulator() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0111_1101,
            Opcode::ROL_Accumulator as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b1111_1010);
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_rol_zeroflag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1000_0000,
            Opcode::ROL_Accumulator as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0000_0000);
        assert!(cpu.get_carry_flag());
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_rol_zeropage() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1000_0000,
            Opcode::ROL_Accumulator as u8, // set carry
            Opcode::ROL_ZeroPage as u8,
            0x7E,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x7E, 0b0000_1001);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x7E), 0b0001_0011);
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_rol_zeropage_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1000_0000,
            Opcode::ROL_Accumulator as u8, // set carry
            Opcode::LDX_Immediate as u8,
            0x03,
            Opcode::ROL_ZeroPageX as u8,
            0x7B,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x7E, 0b0000_1001);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x7E), 0b0001_0011);
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_rol_absolute() {
        let ops = vec![
            Opcode::ROL_Absolute as u8,
            0x32,
            0x18,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x1832, 0b1100_1001);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x1832), 0b1001_0010);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_rol_absolute_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x04,
            Opcode::ROL_AbsoluteX as u8,
            0x2E,
            0x1E,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x1E32, 0b1100_1001);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x1E32), 0b1001_0010);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_ror_accumulator() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1111_1101,
            Opcode::ROR_Accumulator as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0111_1110);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ror_zeroflag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b0000_0001,
            Opcode::ROR_Accumulator as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0b0000_0000);
        assert!(cpu.get_carry_flag());
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ror_zeropage() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1000_0000,
            Opcode::ROL_Accumulator as u8, // set carry
            Opcode::ROR_ZeroPage as u8,
            0x7E,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x7E, 0b0010_1000);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x7E), 0b1001_0100);
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_ror_zeropage_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1000_0000,
            Opcode::ROL_Accumulator as u8, // set carry
            Opcode::LDX_Immediate as u8,
            0x03,
            Opcode::ROR_ZeroPageX as u8,
            0x7B,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x7E, 0b0000_1001);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x7E), 0b1000_0100);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_ror_absolute() {
        let ops = vec![
            Opcode::ROR_Absolute as u8,
            0x32,
            0x0E,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x0E32, 0b1100_1001);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x0E32), 0b01100100);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_ror_absolute_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x04,
            Opcode::ROR_AbsoluteX as u8,
            0x2E,
            0x1A,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x1A32, 0b1100_1001);
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x1A32), 0b01100100);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_rti_implied() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x0C,
            Opcode::PHA_Implied as u8,
            Opcode::LDA_Immediate as u8,
            0x80,
            Opcode::PHA_Implied as u8,
            Opcode::LDA_Immediate as u8,
            0x5D,
            Opcode::PHA_Implied as u8,
            Opcode::LDX_Immediate as u8,
            0xAB,
            Opcode::RTI_Implied as u8,
            Opcode::LDX_Immediate as u8, // skipped by RTI
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.status, 0x5D | CPU::STATUS_RESERVED);
        assert_eq!(cpu.register_x, 0xAB);
    }

    #[test]
    fn test_sbc_immediate() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xF0,
            Opcode::SBC_Immediate as u8,
            0xF0,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0xFF);
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_sbc_zeroflag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xFF,
            Opcode::SBC_Immediate as u8,
            0xFE,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_overflow_flag());
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_sbc_zeropage() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1000_0000,
            Opcode::ROL_Accumulator as u8, // set carry
            Opcode::LDA_Immediate as u8,
            0xA2,
            Opcode::SBC_ZeroPage as u8,
            0xBB,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0xBB, 0x12);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x90);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_sbc_zeropage_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xA2,
            Opcode::LDX_Immediate as u8,
            0x04,
            Opcode::SBC_ZeroPageX as u8,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x16, 0xE3);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0xBE);
        assert!(!cpu.get_carry_flag());
        assert!(!cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_sbc_absolute() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x10,
            Opcode::SBC_Absolute as u8,
            0xAB,
            0x16,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x16AB, 0x06);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x09);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_sbc_absolute_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x50,
            Opcode::LDX_Immediate as u8,
            0x04,
            Opcode::SBC_AbsoluteX as u8,
            0xA7,
            0x16,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x16AB, 0x30);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x1F);
        assert!(cpu.get_carry_flag());
        assert!(!cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_sbc_absolute_y() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xA0,
            Opcode::LDY_Immediate as u8,
            0x04,
            Opcode::SBC_AbsoluteY as u8,
            0xB7,
            0x17,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write(0x17BB, 0x50);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x4F);
        assert!(cpu.get_carry_flag());
        assert!(cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_sbc_indirect_x() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0b1000_0000,
            Opcode::ROL_Accumulator as u8, // set carry
            Opcode::LDA_Immediate as u8,
            0xA0,
            Opcode::LDX_Immediate as u8,
            0x04,
            Opcode::SBC_IndirectX as u8,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x16, 0x132B);
        cpu.mem_write(0x132B, 0x51);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x4F);
        assert!(cpu.get_carry_flag());
        assert!(cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_sbc_indirect_y() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xA0,
            Opcode::LDY_Immediate as u8,
            0x04,
            Opcode::SBC_IndirectY as u8,
            0x12,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x12, 0x132B);
        cpu.mem_write(0x132F, 0x50);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x4F);
        assert!(cpu.get_carry_flag());
        assert!(cpu.get_overflow_flag());
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_sec_implied() {
        let ops = vec![Opcode::SEC_Implied as u8, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert!(cpu.get_carry_flag());
    }

    #[test]
    fn test_sed_implied() {
        let ops = vec![Opcode::SED_Implied as u8, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert!(cpu.get_decimal_mode_flag());
    }

    #[test]
    fn test_sei_implied() {
        let ops = vec![Opcode::SEI_Implied as u8, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert!(cpu.get_interrupt_disable_flag());
    }

    #[test]
    fn test_sta_zeropage() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x3D,
            Opcode::STA_ZeroPage as u8,
            0x1a,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x3D);
        assert_eq!(cpu.mem_read(0x1a), 0x3D);
    }

    #[test]
    fn test_sta_zeropage_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x31,
            Opcode::LDA_Immediate as u8,
            0x3E,
            Opcode::STA_ZeroPageX as u8,
            0x1A,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x3E);
        assert_eq!(cpu.mem_read(0x4B), 0x3E);
    }

    #[test]
    fn test_sta_absolute() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x3C,
            Opcode::STA_Absolute as u8,
            0x1A,
            0x07,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x3C);
        assert_eq!(cpu.mem_read(0x071A), 0x3C);
    }

    #[test]
    fn test_sta_absolute_x() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x12,
            Opcode::LDA_Immediate as u8,
            0x3B,
            Opcode::STA_AbsoluteX as u8,
            0x1A,
            0x02,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x3B);
        assert_eq!(cpu.mem_read(0x022C), 0x3B);
    }

    #[test]
    fn test_sta_absolute_y() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x13,
            Opcode::LDA_Immediate as u8,
            0x35,
            Opcode::STA_AbsoluteY as u8,
            0x1A,
            0x03,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x35);
        assert_eq!(cpu.mem_read(0x032D), 0x35);
    }

    #[test]
    fn test_sta_indirect_x() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x03,
            Opcode::LDA_Immediate as u8,
            0x32,
            Opcode::STA_IndirectY as u8,
            0x87,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x87, 0x34);
        cpu.mem_write_u16(0x88, 0x16);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x32);
        assert_eq!(cpu.mem_read(0x1637), 0x32);
    }

    #[test]
    fn test_sta_indirect_y() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x07,
            Opcode::LDA_Immediate as u8,
            0x35,
            Opcode::STA_IndirectX as u8,
            0x80,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.mem_write_u16(0x87, 0x34);
        cpu.mem_write_u16(0x88, 0x16);
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0x35);
        assert_eq!(cpu.mem_read(0x1634), 0x35);
    }

    #[test]
    fn test_stx_zeropage() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x17,
            Opcode::STX_ZeroPage as u8,
            0x80,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x80), 0x17);
    }

    #[test]
    fn test_stx_zeropage_y() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x17,
            Opcode::LDY_Immediate as u8,
            0x04,
            Opcode::STX_ZeroPageY as u8,
            0x80,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x84), 0x17);
    }

    #[test]
    fn test_stx_absolute() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x17,
            Opcode::STX_Absolute as u8,
            0xA3,
            0x03,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x03A3), 0x17);
    }

    #[test]
    fn test_sty_zeropage() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x17,
            Opcode::STY_ZeroPage as u8,
            0x80,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x80), 0x17);
    }

    #[test]
    fn test_sty_zeropage_x() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x17,
            Opcode::LDX_Immediate as u8,
            0x11,
            Opcode::STY_ZeroPageX as u8,
            0x80,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x91), 0x17);
    }

    #[test]
    fn test_sty_absolute() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x17,
            Opcode::STY_Absolute as u8,
            0x6A,
            0x07,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.mem_read(0x076A), 0x17);
    }

    #[test]
    fn test_tax_implied() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x0a,
            Opcode::TAX_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 10);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_tax_zeroflag() {
        let ops = vec![Opcode::TAX_Implied as u8, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_tax_negative_flag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xFA,
            Opcode::TAX_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0xFA);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_tay_implied() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0x0a,
            Opcode::TAY_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 10);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_tay_zeroflag() {
        let ops = vec![Opcode::TAY_Implied as u8, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_tay_negative_flag() {
        let ops = vec![
            Opcode::LDA_Immediate as u8,
            0xFA,
            Opcode::TAY_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_y, 0xFA);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_tsx_implied() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x0a,
            Opcode::TXS_Implied as u8,
            Opcode::LDX_Immediate as u8,
            0x11,
            Opcode::TSX_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0x0A);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_tsx_zeroflag() {
        let ops = vec![
            Opcode::TXS_Implied as u8,
            Opcode::LDX_Immediate as u8,
            0x11,
            Opcode::TSX_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0x00);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_tsx_negative_flag() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0xF2,
            Opcode::TXS_Implied as u8,
            Opcode::LDX_Immediate as u8,
            0x11,
            Opcode::TSX_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_x, 0xF2);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_txa_implied() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x0a,
            Opcode::TXA_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 10);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_txa_zeroflag() {
        let ops = vec![Opcode::TXA_Implied as u8, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_txa_negative_flag() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0xFA,
            Opcode::TXA_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0xFA);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_txs_implied() {
        let ops = vec![
            Opcode::LDX_Immediate as u8,
            0x0a,
            Opcode::TXS_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();

        // pop values pushed by BRK
        cpu.stack_pop();
        cpu.stack_pop_u16();

        assert_eq!(cpu.stack_pointer, 0x0a);
    }

    #[test]
    fn test_tya_implied() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0x0a,
            Opcode::TYA_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();

        assert_eq!(cpu.register_a, 10);
        assert!(!cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_tya_zeroflag() {
        let ops = vec![Opcode::TYA_Implied as u8, Opcode::BRK_Implied as u8];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0);
        assert!(cpu.get_zero_flag());
        assert!(!cpu.get_negative_flag());
    }

    #[test]
    fn test_tya_negative_flag() {
        let ops = vec![
            Opcode::LDY_Immediate as u8,
            0xFA,
            Opcode::TYA_Implied as u8,
            Opcode::BRK_Implied as u8,
        ];
        let mut cpu = CPU::new(Bus::new(Cartridge::from_opcodes(&ops)));
        cpu.reset_and_run();
        assert_eq!(cpu.register_a, 0xFA);
        assert!(!cpu.get_zero_flag());
        assert!(cpu.get_negative_flag());
    }

    #[test]
    fn test_nestest() {
        let mut cpu = CPU::new(Bus::new(Cartridge::from_file("../nestest.nes").unwrap()));

        cpu.register_a = 0;
        cpu.register_x = 0;
        cpu.register_y = 0;
        cpu.status = 0x24;
        cpu.stack_pointer = 0xFD;
        cpu.program_counter = 0xC000;

        let mut lines: Vec<String> = Vec::new();

        cpu.run_with_callback(|cpu| {
            lines.push(cpu.trace());
        });

        let actual: String = lines.join("\n") + "\n";
        let _ = fs::write("nestest-actual.log", actual);
    }
}
