/**
 * Memory Management System
 * Handles memory allocation and deallocation
 */

export interface MemoryBlock {
  address: number;
  size: number;
  allocated: boolean;
  pid: number | null;
}

export class MemoryManager {
  private totalMemory: number;
  private usedMemory: number = 0;
  private blocks: Map<number, MemoryBlock> = new Map();
  private nextAddress: number = 0;

  constructor(totalMemory: number = 512 * 1024 * 1024) { // 512MB virtual memory
    this.totalMemory = totalMemory;
  }

  allocate(size: number, pid: number): number | null {
    if (this.usedMemory + size > this.totalMemory) {
      return null; // Out of memory
    }

    const address = this.nextAddress;
    this.nextAddress += size;

    const block: MemoryBlock = {
      address,
      size,
      allocated: true,
      pid,
    };

    this.blocks.set(address, block);
    this.usedMemory += size;

    return address;
  }

  deallocate(address: number): boolean {
    const block = this.blocks.get(address);
    if (!block || !block.allocated) {
      return false;
    }

    this.usedMemory -= block.size;
    block.allocated = false;
    block.pid = null;
    this.blocks.delete(address);

    return true;
  }

  deallocateProcess(pid: number): void {
    for (const [address, block] of this.blocks.entries()) {
      if (block.pid === pid) {
        this.deallocate(address);
      }
    }
  }

  getUsage(): { total: number; used: number; free: number } {
    return {
      total: this.totalMemory,
      used: this.usedMemory,
      free: this.totalMemory - this.usedMemory,
    };
  }

  getProcessMemory(pid: number): number {
    let total = 0;
    for (const block of this.blocks.values()) {
      if (block.pid === pid && block.allocated) {
        total += block.size;
      }
    }
    return total;
  }
}
