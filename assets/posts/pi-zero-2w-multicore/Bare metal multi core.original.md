Okay, so [now that we have the single core firmware for the Raspberry Pi Zero 2w working](https://amrishhallberg.com/2026/10/15/blog-1.html), lets get all four cores running. Why is four cores better than one? We have many tasks that we may want to complete with parallism, rather than just concurrency, in an embedded context. Picture a system where you have flight controls, radar imaging, detonation controls, etc. You may not want all of those in your critical loop. If radar processing takes an unexpected amount of time, we don't want our critical loop to stall. There are multiple ways to handle that, such as watchdogs, but multiple parallel cores opens up a lot of cool things we can do, and imposes little overhead, granted we don't race ourselves into oblivion


First things first, all cores share the same memory addresses, because there is no OS underneath to abstract or restrict the space away from us. Remember, unless we write a filesystem, there isn't one. There is nowhere to put data outside of RAM. We have the RPI-given 512-MB of memory. All four cores can see all 512-MB of memory. We need to give each of them their own stack to start so they don't step on each others toes. 

IGU - stands for insert git update. Instead of copying the code into this everytime, the git will have the list of changes, and codex will replace this with the actual code

Okay, now we have 16-KB of memory reserved for our four core stack. Stacks grow down, so we grab their lowest address first, increment by 4K, and grab their highest possible address. 

![[rpi-zero2-four-stack-memory.png]]

Now we can see that they each have their own 4KB to play with, and will not step on each others toes (unless we fuckup. Don't fuck up).

We need to initialize their stacks now, because we currently only handle core 0. Core 0 also has some extra responsibilities that we don't want to race, so we need to jmp the other cores to their own initialization procedure. Here I called it init_stack, and the general flow is:
1. we need to find the stack top. This is labeled in our linker, and is the highest address of the core 0 stack.
2. Because we calculate the other stack by just incrementing 4096 bytes each time, we can recalculate here once we know that starting address. For any given core, number N, we can get the top stack address with
N stack top = (core 0 stack top address) + ( N * 4096)
which we place in the x0 register. Remember, each core has its own registers, so each core has its own x0. We are placing the stack location of each core's stack in their own x0, and then moving that into each of their own, unshared stack pointer. Then each core has a stack pointer register that knows where it starts (and from which we can compute an end).
3. Right now we are then hanging the core again


IGU

Now, that start entry is really only for core 0. We will actually give each of the other cores their own entry, which we can just call something like secondary_entry.
Remember, each core needs the mrs instruction to inform the core who they are. 
The mpidr_el register will tell us our core number, like a "whatsmyip" for a core. Then remember we apply the AND to get the lowest 8 bits in the x1 register. 
Because only core 0 uses ``_start`` now we can remove the jmp to init_stack. 

IGU

Misspelled secondary (insert dog meme)

IGU

Now, we need core 0 to write the entry addresses into the correct slot, and we need it to send them a wake event.
![[Pasted image 20261106085046.png]]
We can see in their official examples that they use 0xe0, 0xe8, and 0xf0 for the cores. 
Lets start by just unleashing one extra core 
(release the hounds meme)

Now, because core 0 is the chosen one and has a super special entry, we can use it to prime the other cores. We branch from bss_done to the kernel_main in our C file, so immediately prior to branching there is a good place to put the multi-core initialization. 

So first we need to put the location of the entry function into the x0 register. Then we put the 0xe0 location that we stole from the RPi foundations homework, into x1. Next we store what is in x0, into the address pointed to by x1. dsb is a special ARM instruction that stands for Data Synchronization Barrier,  and forces the cpu to wait for all loads, stores, cache ops, and translation lookaside buffer operations to be done. This ensures we don't do anything before the values we expect are actually instantiated. If we don't, a worker could wake up and read its address before it has a value other than 0, and go back to sleep. sev is the Send Event instruction. 

IGU

Alright cool, now its trivial to get each of the other two up and running as well

IGU

If this has gotten a little confusing, that is perfectly okay![[rpi-zero2-multicore-startup-walkthrough.png]]
Hopefully this graphic can help you trace the logic of each core.
Now, each core is theoretically alive, but right now we are just hanging them and would need electrical traces to determine if they were actually doing something. In order to get some sort of signal, we can take advantage of the fact that they all share the same RAM, and give them unique derived locations to write to, as a "flag", indicating that they are indeed working as intended. This is the first thing we have really done where we need to be worried about any kind of interference from other cores. How do we make sure that they don't accidentally overwrite each others data? We can use their unique core numbers, so that even running the same calculation they can never derive the same memory address. 

We can follow an algorithm something roughly like 
flag address = flag base + (core n * 8)
now, we need somewhere to put this new 32 bytes worth of flag memory.
.section .bss changes our mode from executable instructions to uninitialized storage. "aw" means allocateable and writeable. @nobits means we dont have inital data to go into the range. 
.balign 8 aligns us to the 8 byte boundary
We  are just calling this memory region core_alive because that is what we are checking
.skip 32 reserves our 32 bytes (8 bytes for 4 cores)
if we run through our flag address algorithm
core_alive +  0   core 0’s slot
core_alive +  8   core 1’s slot
core_alive + 16   core 2’s slot
core_alive + 24   core 3’s slot

IGU

Now we need to have each worker locate its own slot in the core_alive section, and to store a "1" there, so we know they completed their procedure.We'll put that in init_stack. First we want to put the memory address of core_alive into one of their registers, so they know where to write. Then we want to calculate their writing spot. lsl 3 means shift left by 3 bits, which is the same thing as multiplying a number by 2^3 or 8. for example if our bits start at ``00000010`` and we shift left 3 bits, we get ``00010000`` , or in other words we started with 2 and ended with 16 (a multiplicative of 8). 

IGU

So now x2 holds our intended write address for each core, we need to write 1. We can just use x3 for this, and we can store the value in x3 into the memory indicated by the address in x2. 

IGU

now lets go back to core 0. So each core 1-3 SHOULD have, by now, awoken, initialized its stack, determined where its core_alive reservation is, written 1, and hung. Core 0 should check these to ensure all the cores are alive.  Remember core 0 doesnt need to write there, it just needs to read it. So we get the address of core_alive, add 8 so we are looking at core 1's slot, and then we read the data in the x2 memory address into our x3 register. If x3 = 0, the core is not alive yet. If x3 = 1, the core is alive. 
```    
    ldr x2, =core_alive
    add x2, x2, #8
    ldar x3, [x2]
```
(its alive meme)

now, we need that to be reusable, so instead of placing it in a specific place in our path, lets make it a new procedure. Remember, in proper languages (which are exclusively assembly and HDL variants), where things go in a file matters, so the procedure needs to be inside of the executable .text section, before the .bss section. 
So we are going to pass in the argument for which core we are looking for in x0. 
1. We read the core_alive address into x2
2. we increment x2 by 8, x0 number of times. 
3. we then read the value in that 8 byte memory region into x0, which is our return
4. we return
so now our function will either return 0 (core is not alive) or 1 (core is alive)

We also need to add the read_core_alive function to our main.c, so it is expecting that definition somewhere (which comes from our boot.S).

IGU


We need to give ourselves some capabilities most OS's would provide in order to not be bound by program execution in terms of things like time tracking. We need to know two main things:
1. What is the timer tick value?
2. How many ticks are one second?
This lets our counter run independently of our polling loop, which helps us keep real-time deadlines. 

For the first helper, getting the current tick value, we want use the ``mrs`` instruction to query the **Counter-timer Physical Count Register**. ISB is the instruction synchronization barrier, and ensures the read comes after earlier instructions, so we have an accurate timer change assessment.

Now in the other helper, we want to read the frequncy of the system in Hz, so we know how to do our frequncy calculations. This is called the Counter-timer Frequency Register.

IGU

Okay, now to our kernel_main, we need to add the checking. We need to figure out how long one second is, what time we are starting at, and then we need to start polling the flags. You may notice there is a second condition beyond all the flags being set, and that is that the elapsement of timer ticks has been less than one seconds worth. This is a simple example (bounded polling) of how we can meet hard, real-time deadlines with our embedded systems. If it takes longer than one second, something is very, very wrong. Imagine if this was your car, and the car systems are busy doing a bunch of calculations. There is only so long we can allow that before we need to get it back to paying attention to your breaking, steering, air bag deployment, etc. otherwise your car won't respond and you will have a pink panther kind of day on the road. 

IGU

Now that we are reading them, we want to give our console some sort of validation that the cores are awake. So we will reuse the uart_puts to do it. For each core we will just use a simple ternary operator to put one string if true, and one string if false. 

IGU

Now, we have successful output, but I want to know how long startup took. This will help us have an idea of how long it takes to go from applying power to doing that things that we want. So lets add a uart_put_ulong in uart.c and uart.h so we can put unsigned long numbers to the console. 20 characters fit the biggest 64-bit unsigned decimal value. the addition of '0' to the value converts it into the proper character, and we can take that character and put it to the serial console now with uart_putc.


IGU



Okay, we are past a lot of the scaffolding, and we need to actually give the cores something to do other than hang. so lets replace the b hang with something that enters them into a function. Currently we have the registers identify in x1, and the ARM64 ABI says we need to pass our first argument in x0, so we need to move that first. Then we branch with link to a function I'm calling secondary_main. 
So now we need to define secondary_main.

IGU


I think the best way if we have given functions in mind, is to use a case statement, and assign each core a function based on its core number. 


IGU


Okay, so this should technically work but we have not actually defined anything that they should be doing. We can make a small token function to just have them doing work. 

IGU

Now, each of these core is currently just writing into their own stack, and that is private. That means our core 0 can't actually access them and tell us any of the values. So instead, I would like to reserve another set of memory address, offset by a size increment times the core number. Once again, this stops the cores from racing, as we can ensure core 1 never writes into core 2's space, core 2 never writes into core 1 or 3's space, and so on. 
Lets go back into boot.S and add another .bss section. Before we add another toy size section, we should remember this will be a race-free section of essentially "shared" memory, where workers can place and move data around for each other, separate from their private stack. We should also remember that the stack addresses follow the bss section, and will move according to the amount we allocate. I think 64KB per core is a decent place to start, because it gives us a fair amount of memory to use. Now when I say "shared" here, I expect one core to be able to write to one of the four sections, but it is able to read the other 3. We absolutely do not want all 4 cores writing to all four sections of this reserved memory. There is no kernel protection against such writes here (well, we ARE the kernel protection I suppose). So now each core can determine the start of its allowed address by finding 
allowed_region = core_shared + (core N * 65536)
and they can write up to 64KB-1 bytes to that section, before they would bleed into their neighbor core's area.  


IGU

With that extra memory region now, we should modify the counter function and add a proper assembly helper that ensures we only write to the correct region, and that we only write the first 8 bytes. 
1. First, we will declare write_core_counter as an actual function that can be located
2. Then we load the core_shared address into x2
3. Then we apply the allowed_region = core_shared + (core N * 65536) formula, shifting 16 bits left to multiply by 64KB (2^10 is 1KB, 2^6 is 64, 2^16=64KB)
4. Then we store the value in x1 into the memory address pointed to by x2
5. return 

now, it also needs to be declared in main.c so that we know its defined somewhere at link time. Because each worker was just giving their own core number, only taking the step worked, but now we need to pass in the core number so each time the function is ran, it knows what region it is allowed to write into. 
We can adjust the counter_worker inputs, and also ensure we use the assembly writer to actually write to the assigned memory.

IGU

Now, we also need to allow core 0 some way to read and interpret those writes and values. Remember, in this ABI, x0 will have argument 1. We want the first argument to be the core number. When we do our add and lsl, we will do it by the appropriate core number. So lets
1. Declare our read_core_counter function
2. Load the core_shared address into its x2 register
3. shift left by the appropriate 16 bits
4. load the value the x2 address points to into x0
5. and use that value as the return value
We need to declare this function as well in main.c

IGU

Now, we dont want to clutter our command line real bad with these values coming at us all the time. It would be better if our shell only printed these when we input something like "core-counters". For that, we need some way to tell if our input value into the shell matches some value we predefined in our exe. Lets add a very simple strings_equal function, and lets add the printing our on console so we can see it. 

IGU

Alright, now, we haven't validated any of this yet because I wrote it between CDL and JFK, and pulling microcomputers and wires out on planes is frowned upon. Lets compile, examine, and test this

``clang --target=aarch64-none-elf -c platform/multicore/boot.S -o build/multicore/boot.o``

``` 
clang --target=aarch64-none-elf -c platform/multicore/boot.S -o build/multicore/boot.o


clang --target=aarch64-none-elf -std=c11 -ffreestanding -fno-builtin -fno-stack-protector -fno-unwind-tables -fno-asynchronous-unwind-tables -fno-pic -O0 -Wall -Wextra -Iplatform/multicore -c platform/multicore/main.c -o build/multicore/main.o


clang --target=aarch64-none-elf -std=c11 -ffreestanding -fno-builtin -fno-stack-protector -fno-unwind-tables -fno-asynchronous-unwind-tables -fno-pic -O0 -Wall -Wextra -Iplatform/multicore -c platform/multicore/uart.c -o build/multicore/uart.o

ld.lld -m aarch64elf -T platform/multicore/linker.ld -o build/multicore/kernel8.elf build/multicore/boot.o build/multicore/main.o build/multicore/uart.o

llvm-objcopy -O binary build/multicore/kernel8.elf build/multicore/kernel8.img


```

![[Pasted image 20261106121644.png]]

okay lets inspect it
```
llvm-objdump -d build/multicore/kernel8.elf 
```

Our timer functions look correct:

![[Pasted image 20261106121827.png]]

counter worker and its dispatcher look good:

![[Pasted image 20261106121911.png]]
![[Pasted image 20261106121937.png]]

and our shared memory functions look correct for reading and writing:
![[Pasted image 20261106122043.png]]

Beautiful. Let's actually run it and see what breaks. Remember to copy the newly compiled kernel8.img to the raspberry pi's SD card. 
![[Pasted image 20261106122636.png]]
setup or COM session
![[Pasted image 20261106175047.png]]
alright, i could have used something with finer resolution than a millisecond. That's not very helpful. however, we have what we were looking for.
![[Pasted image 20261106175147.png]]
Okay, so we now effectively have the boot sequence, core initialization, and how we assign different cores different memory regions and functions. Now we can go into more useful and specialized functions. 