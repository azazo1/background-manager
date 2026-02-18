export interface Task {
  id?: number;
  name: string;
  program: string;
  working_dir?: string;
  args: string[];
  stdin?: string;
  stdout?: string;
  stderr?: string;
  trigger: Trigger;
  enabled: boolean;
  no_console?: boolean;
  last_exit_code?: number;
  last_run_at?: string;
}

export type Trigger =
  | { tag: "Routine"; content: number }
  | { tag: "Instant"; content: string }
  | { tag: "Startup" }
  | { tag: "KeepAlive" }
  | { tag: "Manual" }
  | { tag: "UntilSucceed" };

export interface TaskRunStatus {
  id: number;
  is_running: boolean;
  last_exit_code?: number;
  last_run_at?: string;
}

export enum TaskStatus {
  Suspended = "Suspended",
  Running = "Running",
  Idle = "Idle"
}
