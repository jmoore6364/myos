/**
 * System Call Interface
 * Provides the API for user programs to interact with the kernel
 */

import type { Kernel } from './core';

export class SystemCalls {
  constructor(private kernel: Kernel) {}

  // Process management
  async exec(command: string, args: string[] = []): Promise<number> {
    return this.kernel.exec(command, args);
  }

  async fork(): Promise<number> {
    const currentProcess = this.kernel.processManager.getCurrentProcess();
    if (!currentProcess) {
      throw new Error('No current process');
    }

    // Create a child process
    const pid = this.kernel.processManager.createProcess(
      `fork-${currentProcess.getContext().name}`,
      async () => {}, // Empty task, will be replaced by caller
      currentProcess.getContext().pid
    );

    return pid;
  }

  exit(code: number = 0): void {
    const currentProcess = this.kernel.processManager.getCurrentProcess();
    if (currentProcess) {
      currentProcess.kill();
    }
  }

  kill(pid: number): boolean {
    return this.kernel.processManager.killProcess(pid);
  }

  // File system operations
  async readFile(path: string): Promise<string> {
    return this.kernel.fs.readFile(path);
  }

  async writeFile(path: string, content: string): Promise<void> {
    return this.kernel.fs.writeFile(path, content);
  }

  async deleteFile(path: string): Promise<void> {
    return this.kernel.fs.deleteFile(path);
  }

  async listDirectory(path: string): Promise<string[]> {
    return this.kernel.fs.listDirectory(path);
  }

  async createDirectory(path: string): Promise<void> {
    return this.kernel.fs.createDirectory(path);
  }

  async stat(path: string): Promise<any> {
    return this.kernel.fs.stat(path);
  }

  // Memory operations
  malloc(size: number): number | null {
    const currentProcess = this.kernel.processManager.getCurrentProcess();
    if (!currentProcess) {
      return null;
    }

    return this.kernel.memoryManager.allocate(size, currentProcess.getContext().pid);
  }

  free(address: number): boolean {
    return this.kernel.memoryManager.deallocate(address);
  }

  // I/O operations
  async print(message: string): Promise<void> {
    this.kernel.emit('stdout', message);
  }

  async println(message: string): Promise<void> {
    this.kernel.emit('stdout', message + '\n');
  }

  async input(prompt?: string): Promise<string> {
    if (prompt) {
      await this.print(prompt);
    }

    return new Promise((resolve) => {
      this.kernel.once('stdin', (data: string) => {
        resolve(data);
      });
    });
  }

  // System information
  getSystemInfo(): any {
    return {
      version: '0.1.0',
      uptime: Date.now() - this.kernel.bootTime,
      memory: this.kernel.memoryManager.getUsage(),
      processes: this.kernel.processManager.listProcesses().length,
    };
  }

  // Time operations
  getCurrentTime(): number {
    return Date.now();
  }

  async sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
