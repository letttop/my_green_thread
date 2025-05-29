use core::arch::asm;

// stack size
const SSIZE: isize = 48;

// only one for now
// but need all callee to save later
#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    sp: u64,
}

fn hello() -> ! {
    println!("I LOVE WAKING UP ON A NEW STACK!");
    loop {}
}

fn gt_switch(new: *const ThreadContext) {
    unsafe {
        asm!(
            "ld sp, 0({0})",// {0} 指向第一个输入操作数，也就是reg
            "ld ra, 0(sp)",
            "ret",  // jalr rd, rs1, imm ：保存pc+4到rd，跳转imm(rs1); ret = jalr x0, x1, 0, 就是直接跳转x1，x1=ra，
            in(reg) new,    // 这里的reg由asm!决定，任意临时寄存器
            options(noreturn)
        );
    }
}

fn main() {
    let mut ctx = ThreadContext::default();
    let mut stack = vec![0_u8; SSIZE as usize];
    unsafe {
        // 栈底，就是最高位
        let stack_bottom = stack.as_mut_ptr().offset(SSIZE);
        // !15=111...1110000
        let sb_aligned = (stack_bottom as usize & !15) as *mut u8;
        // casting function pointer `hello` to `u64`并写入栈内
        // 此时hello在.text段
        std::ptr::write(sb_aligned.offset(-8) as *mut u64, hello as usize as u64);
        // 向下回到栈顶（仅保存，在gt_switch中跳转）
        ctx.sp = sb_aligned.offset(-8) as u64;
        // 当调用函数时，会
        //  保存caller寄存器
        //  设置a0-a7作为B的输入参数
        //  设置ra为A的PC+4——————————此时pc指向gt_switch
        //  分配B的栈空间
        //  保存B要返回的地址ra
        //  保存callee寄存器
        //  运行
        gt_switch(&ctx);
    }
}
