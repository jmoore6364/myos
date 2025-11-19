/**
 * Process Management System
 * Handles process creation, scheduling, and lifecycle management
 */

export enum ProcessState {
  READY = 'READY',
  RUNNING = 'RUNNING',
  BLOCKED = 'BLOCKED',
  TERMINATED = 'TERMINATED',
}

export enum ProcessPriority {
  SYSTEM = 0,
  HIGH = 1,
  NORMAL = 2,
  LOW = 3,
}

export interface ProcessContext {
  pid: number;
  parentPid: number | null;
  name: string;
  state: ProcessState;
  priority: ProcessPriority;
  createdAt: number;
  cpuTime: number;
  memory: number;
  env: Record<string, string>;
  cwd: string;
  stdin?: ReadableStream<string>;
  stdout?: WritableStream<string>;
  stderr?: WritableStream<string>;
}

export class Process {
  private context: ProcessContext;
  private task: (() => Promise<void>) | null = null;
  private controller: AbortController;

  constructor(
    pid: number,
    name: string,
    task: () => Promise<void>,
    parentPid: number | null = null,
    priority: ProcessPriority = ProcessPriority.NORMAL
  ) {
    this.controller = new AbortController();
    this.context = {
      pid,
      parentPid,
      name,
      state: ProcessState.READY,
      priority,
      createdAt: Date.now(),
      cpuTime: 0,
      memory: 0,
      env: { ...globalThis.process?.env } || {},
      cwd: '/',
    };
    this.task = task;
  }

  async run(): Promise<void> {
    this.context.state = ProcessState.RUNNING;
    const startTime = performance.now();

    try {
      if (this.task) {
        await this.task();
      }
      this.context.state = ProcessState.TERMINATED;
    } catch (error) {
      console.error(`Process ${this.context.pid} error:`, error);
      this.context.state = ProcessState.TERMINATED;
      throw error;
    } finally {
      const endTime = performance.now();
      this.context.cpuTime += endTime - startTime;
    }
  }

  suspend(): void {
    if (this.context.state === ProcessState.RUNNING) {
      this.context.state = ProcessState.BLOCKED;
    }
  }

  resume(): void {
    if (this.context.state === ProcessState.BLOCKED) {
      this.context.state = ProcessState.READY;
    }
  }

  kill(): void {
    this.controller.abort();
    this.context.state = ProcessState.TERMINATED;
  }

  getContext(): ProcessContext {
    return { ...this.context };
  }

  getSignal(): AbortSignal {
    return this.controller.signal;
  }

  updateCwd(newCwd: string): void {
    this.context.cwd = newCwd;
  }

  setEnv(key: string, value: string): void {
    this.context.env[key] = value;
  }
}

export class ProcessManager {
  private processes: Map<number, Process> = new Map();
  private nextPid: number = 1;
  private runningProcess: Process | null = null;

  createProcess(
    name: string,
    task: () => Promise<void>,
    parentPid: number | null = null,
    priority: ProcessPriority = ProcessPriority.NORMAL
  ): number {
    const pid = this.nextPid++;
    const process = new Process(pid, name, task, parentPid, priority);
    this.processes.set(pid, process);
    return pid;
  }

  async executeProcess(pid: number): Promise<void> {
    const process = this.processes.get(pid);
    if (!process) {
      throw new Error(`Process ${pid} not found`);
    }

    this.runningProcess = process;
    try {
      await process.run();
    } finally {
      this.runningProcess = null;
      // Clean up terminated processes
      if (process.getContext().state === ProcessState.TERMINATED) {
        this.processes.delete(pid);
      }
    }
  }

  killProcess(pid: number): boolean {
    const process = this.processes.get(pid);
    if (!process) {
      return false;
    }

    process.kill();
    this.processes.delete(pid);
    return true;
  }

  getProcess(pid: number): Process | undefined {
    return this.processes.get(pid);
  }

  listProcesses(): ProcessContext[] {
    return Array.from(this.processes.values()).map(p => p.getContext());
  }

  getCurrentProcess(): Process | null {
    return this.runningProcess;
  }
}
