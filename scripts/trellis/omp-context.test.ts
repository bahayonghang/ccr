import { afterAll, describe, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import registerTrellis from "../../.omp/extensions/trellis/index.ts";

const PRD_MARKER = "OMP_CTX_PRD_MARKER";
const DESIGN_MARKER = "OMP_CTX_DESIGN_MARKER";
const IMPLEMENT_MD_MARKER = "OMP_CTX_IMPLEMENT_MD_MARKER";
const INFO_MARKER = "OMP_CTX_INFO_MARKER";
const IMPLEMENT_JSONL_MARKER = "OMP_CTX_IMPLEMENT_JSONL_MARKER";
const CHECK_JSONL_MARKER = "OMP_CTX_CHECK_JSONL_MARKER";
const UNTRUSTED_MARKER = "OMP_CTX_UNTRUSTED_MARKER";

const SESSION_ID = "review-fixture";
const COMPLEX_TASK = ".trellis/tasks/fixture-complex";
const LIGHT_TASK = ".trellis/tasks/fixture-light";
const MISSING_TASK = ".trellis/tasks/fixture-missing";

type TrellisRole = "trellis-implement" | "trellis-check" | "trellis-research";
type SessionRole = TrellisRole | "main";

const ownedRoot = mkdtempSync(join(tmpdir(), "ccr-omp-context-"));
const complexRepo = join(ownedRoot, "complex-repo");
const lightRepo = join(ownedRoot, "light-repo");
const missingRepo = join(ownedRoot, "missing-repo");
const outsideSecret = join(ownedRoot, "outside-secret.md");

function writeUtf8(path: string, content: string): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content, "utf-8");
}

function writeSession(repo: string, currentTask: string): void {
  writeUtf8(
    join(repo, ".trellis", ".runtime", "sessions", "omp_review-fixture.json"),
    `${JSON.stringify({ current_task: currentTask }, null, 2)}\n`,
  );
}

function writeTaskJson(repo: string, taskRel: string, title: string): void {
  writeUtf8(
    join(repo, taskRel, "task.json"),
    `${JSON.stringify({ status: "in_progress", title }, null, 2)}\n`,
  );
}

function expectedJsonl(role: SessionRole): { include: string[]; exclude: string[] } {
  switch (role) {
    case "main":
      return {
        include: [IMPLEMENT_JSONL_MARKER, CHECK_JSONL_MARKER],
        exclude: [UNTRUSTED_MARKER],
      };
    case "trellis-implement":
      return {
        include: [IMPLEMENT_JSONL_MARKER],
        exclude: [CHECK_JSONL_MARKER, UNTRUSTED_MARKER],
      };
    case "trellis-check":
      return {
        include: [CHECK_JSONL_MARKER],
        exclude: [IMPLEMENT_JSONL_MARKER, UNTRUSTED_MARKER],
      };
    case "trellis-research":
      return {
        include: [],
        exclude: [IMPLEMENT_JSONL_MARKER, CHECK_JSONL_MARKER, UNTRUSTED_MARKER],
      };
    default: {
      const exhaustive: never = role;
      throw new Error(`unhandled session role: ${exhaustive}`);
    }
  }
}

function createMockApi(): {
  api: {
    on: (event: string, handler: (...args: never[]) => unknown) => void;
    sendMessage: (message: {
      customType?: string;
      content?: string;
      display?: boolean;
    }) => Promise<void>;
  };
  handlers: Map<string, (...args: never[]) => unknown>;
  contents: string[];
} {
  const handlers = new Map<string, (...args: never[]) => unknown>();
  const contents: string[] = [];
  return {
    handlers,
    contents,
    api: {
      on(event: string, handler: (...args: never[]) => unknown) {
        handlers.set(event, handler);
      },
      async sendMessage(message: { content?: string }) {
        if (typeof message.content === "string") {
          contents.push(message.content);
        }
      },
    },
  };
}

async function startSession(cwd: string, role: SessionRole) {
  const previousBlocked = process.env.PI_BLOCKED_AGENT;
  const mock = createMockApi();
  try {
    if (role === "main") {
      delete process.env.PI_BLOCKED_AGENT;
    } else {
      process.env.PI_BLOCKED_AGENT = role;
    }

    registerTrellis(mock.api as Parameters<typeof registerTrellis>[0]);
  } finally {
    if (previousBlocked === undefined) {
      delete process.env.PI_BLOCKED_AGENT;
    } else {
      process.env.PI_BLOCKED_AGENT = previousBlocked;
    }
  }

  const sessionStart = mock.handlers.get("session_start");
  if (!sessionStart) {
    throw new Error("session_start was not registered");
  }
  const context = {
    cwd,
    sessionManager: { getSessionId: () => SESSION_ID },
    ui: { notify: () => {} },
  };
  await sessionStart({} as never, context as never);
  return { ...mock, context };
}

async function collectSessionStart(cwd: string, role: SessionRole): Promise<string> {
  const session = await startSession(cwd, role);
  return session.contents.join("\n");
}

function seedTaskRepo(name: string, config = ""): string {
  const repo = join(ownedRoot, name);
  writeSession(repo, COMPLEX_TASK);
  writeTaskJson(repo, COMPLEX_TASK, name);
  writeUtf8(join(repo, COMPLEX_TASK, "prd.md"), `${PRD_MARKER}\n`);
  if (config) writeUtf8(join(repo, ".trellis", "config.yaml"), config);
  return repo;
}

function seedComplexRepo(): void {
  writeSession(complexRepo, COMPLEX_TASK);
  writeTaskJson(complexRepo, COMPLEX_TASK, "OMP complex fixture");
  writeUtf8(join(complexRepo, COMPLEX_TASK, "prd.md"), `${PRD_MARKER}\n`);
  writeUtf8(join(complexRepo, COMPLEX_TASK, "design.md"), `${DESIGN_MARKER}\n`);
  writeUtf8(join(complexRepo, COMPLEX_TASK, "implement.md"), `${IMPLEMENT_MD_MARKER}\n`);
  writeUtf8(join(complexRepo, COMPLEX_TASK, "info.md"), `${INFO_MARKER}\n`);
  writeUtf8(
    join(complexRepo, COMPLEX_TASK, "implement-spec.md"),
    `${IMPLEMENT_JSONL_MARKER}\n`,
  );
  writeUtf8(join(complexRepo, COMPLEX_TASK, "check-spec.md"), `${CHECK_JSONL_MARKER}\n`);
  writeUtf8(
    join(complexRepo, COMPLEX_TASK, "implement.jsonl"),
    [
      JSON.stringify({
        file: `${COMPLEX_TASK}/implement-spec.md`,
        reason: "trusted implement manifest",
      }),
      JSON.stringify({
        file: "../outside-secret.md",
        reason: "path outside the fixture repo",
      }),
      JSON.stringify({ file: outsideSecret, reason: "absolute outside path" }),
      JSON.stringify({ file: "invalid\0path", reason: "invalid path" }),
      "null",
      "{malformed",
      "",
    ].join("\n"),
  );
  writeUtf8(
    join(complexRepo, COMPLEX_TASK, "check.jsonl"),
    `${JSON.stringify({
      file: `${COMPLEX_TASK}/check-spec.md`,
      reason: "trusted check manifest",
    })}\n`,
  );
  writeUtf8(outsideSecret, `${UNTRUSTED_MARKER}\n`);
}

function seedLightRepo(): void {
  writeSession(lightRepo, LIGHT_TASK);
  writeTaskJson(lightRepo, LIGHT_TASK, "OMP light fixture");
  writeUtf8(join(lightRepo, LIGHT_TASK, "prd.md"), `${PRD_MARKER}\n`);
}

function seedMissingRepo(): void {
  writeSession(missingRepo, MISSING_TASK);
  writeTaskJson(missingRepo, MISSING_TASK, "OMP missing fixture");
}

seedComplexRepo();
seedLightRepo();
seedMissingRepo();

afterAll(() => {
  rmSync(ownedRoot, { recursive: true, force: true });
});

describe("OMP Trellis buildTaskContext", () => {
  test("main and each role receive prd/design/implement markers", async () => {
    const roles: SessionRole[] = [
      "main",
      "trellis-implement",
      "trellis-check",
      "trellis-research",
    ];
    for (const role of roles) {
      const content = await collectSessionStart(complexRepo, role);
      expect(content, `${role} missing PRD`).toContain(PRD_MARKER);
      expect(content, `${role} missing Design`).toContain(DESIGN_MARKER);
      expect(content, `${role} missing Implement`).toContain(IMPLEMENT_MD_MARKER);
      for (const artifact of ["prd.md", "design.md", "implement.md"]) {
        expect(content, `${role} missing ${artifact} source`).toContain(
          `### ${COMPLEX_TASK}/${artifact} [inline]`,
        );
      }
    }
  });

  test("jsonl files stay isolated by role", async () => {
    const roles: SessionRole[] = [
      "main",
      "trellis-implement",
      "trellis-check",
      "trellis-research",
    ];
    for (const role of roles) {
      const content = await collectSessionStart(complexRepo, role);
      const { include, exclude } = expectedJsonl(role);
      for (const marker of include) {
        expect(content, `${role} missing ${marker}`).toContain(marker);
      }
      for (const marker of exclude) {
        expect(content, `${role} leaked ${marker}`).not.toContain(marker);
      }
    }
  });

  test("lightweight task without design/implement still reads PRD", async () => {
    const content = await collectSessionStart(lightRepo, "main");
    expect(content).toContain(PRD_MARKER);
    expect(content).toContain(`### ${LIGHT_TASK}/prd.md [inline]`);
    expect(content).not.toContain(DESIGN_MARKER);
    expect(content).not.toContain(IMPLEMENT_MD_MARKER);
    expect(content).not.toContain(`${LIGHT_TASK}/design.md`);
    expect(content).not.toContain(`${LIGHT_TASK}/implement.md`);
  });

  test("missing task files do not crash session_start", async () => {
    const content = await collectSessionStart(missingRepo, "main");
    expect(content).not.toContain(PRD_MARKER);
    expect(content).not.toContain(DESIGN_MARKER);
    expect(content).not.toContain(IMPLEMENT_MD_MARKER);
  });

  test("out-of-trust jsonl paths stay rejected", async () => {
    const content = await collectSessionStart(complexRepo, "main");
    expect(content).toContain(IMPLEMENT_JSONL_MARKER);
    expect(content).not.toContain(UNTRUSTED_MARKER);
  });

  test("legacy info.md remains available to every role", async () => {
    const roles: SessionRole[] = [
      "main", "trellis-implement", "trellis-check", "trellis-research",
    ];
    for (const role of roles) {
      const content = await collectSessionStart(complexRepo, role);
      expect(content).toContain(INFO_MARKER);
      expect(content).toContain(`${COMPLEX_TASK}/info.md [inline]`);
    }
  });

  test("design and implementation edits refresh the existing session cache", async () => {
    const repo = seedTaskRepo("cache-repo");
    const designPath = join(repo, COMPLEX_TASK, "design.md");
    writeUtf8(designPath, DESIGN_MARKER);
    const session = await startSession(repo, "trellis-implement");
    const contextHandler = session.handlers.get("context");
    if (!contextHandler) throw new Error("context was not registered");
    type Message = { customType?: string; content?: string };
    let messages: Message[] = [{
      customType: "trellis-task-context",
      content: session.contents.join("\n"),
    }];
    expect(messages[0].content).toContain(DESIGN_MARKER);

    const refresh = async (): Promise<string> => {
      const updated = await contextHandler(
        { messages } as never, session.context as never,
      ) as { messages: Message[] };
      messages = updated.messages;
      const taskMessages = messages.filter((message) => message.customType === "trellis-task-context");
      expect(taskMessages).toHaveLength(1);
      return taskMessages[0].content ?? "";
    };

    writeUtf8(designPath, "UPDATED_DESIGN_CONTENT");
    const changedDesign = await refresh();
    expect(changedDesign).toContain("UPDATED_DESIGN_CONTENT");
    expect(changedDesign).not.toContain(DESIGN_MARKER);

    writeUtf8(join(repo, COMPLEX_TASK, "implement.md"), IMPLEMENT_MD_MARKER);
    expect(await refresh()).toContain(IMPLEMENT_MD_MARKER);

    writeUtf8(join(repo, COMPLEX_TASK, "implement.md"), "UPDATED_IMPLEMENTATION_CONTENT");
    const changedImplementation = await refresh();
    expect(changedImplementation).not.toContain(IMPLEMENT_MD_MARKER);
    expect(changedImplementation).toContain("UPDATED_IMPLEMENTATION_CONTENT");

    rmSync(designPath);
    const removedDesign = await refresh();
    expect(removedDesign).not.toContain("UPDATED_DESIGN_CONTENT");
    expect(removedDesign).toContain("UPDATED_IMPLEMENTATION_CONTENT");
  });

  test("artifact and referenced-file symlinks cannot read outside trusted roots", async () => {
    const repo = seedTaskRepo("symlink-repo");
    symlinkSync(outsideSecret, join(repo, COMPLEX_TASK, "design.md"), "file");
    symlinkSync(outsideSecret, join(repo, COMPLEX_TASK, "implement.md"), "file");
    symlinkSync(outsideSecret, join(repo, COMPLEX_TASK, "linked-spec.md"), "file");
    writeUtf8(join(repo, COMPLEX_TASK, "implement.jsonl"), JSON.stringify({
      file: `${COMPLEX_TASK}/linked-spec.md`,
      reason: "symlink outside the fixture repo",
    }));
    const content = await collectSessionStart(repo, "trellis-implement");
    expect(content).toContain(PRD_MARKER);
    expect(content).not.toContain(UNTRUSTED_MARKER);
    expect(content).not.toContain(`${COMPLEX_TASK}/design.md`);
    expect(content).not.toContain(`${COMPLEX_TASK}/implement.md`);
    expect(content).not.toContain(`${COMPLEX_TASK}/linked-spec.md`);
  });

  test("explicit trusted roots permit artifact and manifest symlinks", async () => {
    const shared = join(ownedRoot, "trusted-documents");
    writeUtf8(join(shared, "design.md"), DESIGN_MARKER);
    writeUtf8(join(shared, "spec.md"), IMPLEMENT_JSONL_MARKER);
    const repo = seedTaskRepo("trusted-repo",
      "channel:\n  trusted_context_dirs:\n    - ../trusted-documents\n");
    symlinkSync(join(shared, "design.md"), join(repo, COMPLEX_TASK, "design.md"), "file");
    writeUtf8(join(repo, COMPLEX_TASK, "implement.jsonl"), JSON.stringify({
      file: "../trusted-documents/spec.md", reason: "explicit trusted document",
    }));
    const content = await collectSessionStart(repo, "trellis-implement");
    expect(content).toContain(PRD_MARKER);
    expect(content).toContain(DESIGN_MARKER);
    expect(content).toContain(IMPLEMENT_JSONL_MARKER);
  });

  test("artifact and file caps truncate on complete UTF-8 characters", async () => {
    const repo = seedTaskRepo("utf8-repo",
      "context_injection:\n  max_artifact_bytes: 31\n  max_file_bytes: 15\n  max_total_bytes: 2048\n");
    writeUtf8(join(repo, COMPLEX_TASK, "design.md"), `${"A".repeat(30)}设计🔐${"Z".repeat(200)}`);
    writeUtf8(join(repo, COMPLEX_TASK, "spec.md"), `${"B".repeat(14)}é🌐${"F".repeat(200)}`);
    writeUtf8(join(repo, COMPLEX_TASK, "implement.jsonl"), JSON.stringify({
      file: `${COMPLEX_TASK}/spec.md`, reason: "UTF-8 boundary fixture",
    }));
    const content = await collectSessionStart(repo, "trellis-implement");
    expect(content).toContain(`${COMPLEX_TASK}/design.md [truncated]`);
    expect(content).toContain(`${"A".repeat(30)}\n[Trellis: truncated at 31 bytes`);
    expect(content).toContain(`${COMPLEX_TASK}/spec.md [truncated]`);
    expect(content).toContain(`${"B".repeat(14)}\n[Trellis: truncated at 15 bytes`);
    expect(content).not.toContain("�");
    expect(content).not.toContain("设计");
    expect(content).not.toContain("é");
    expect(Buffer.byteLength(content, "utf-8")).toBeLessThanOrEqual(2048);
  });

  test("total context cap emits a required-read notice without oversized content", async () => {
    const repo = seedTaskRepo("budget-repo",
      "context_injection:\n  max_artifact_bytes: 4096\n  max_total_bytes: 700\n");
    writeUtf8(join(repo, COMPLEX_TASK, "design.md"), `${DESIGN_MARKER}${"X".repeat(3000)}`);
    const content = await collectSessionStart(repo, "trellis-implement");
    expect(content).toContain(PRD_MARKER);
    expect(content).toContain(`${COMPLEX_TASK}/design.md [omitted]`);
    expect(content).toContain(`required_read: ${COMPLEX_TASK}/design.md`);
    expect(content).not.toContain(DESIGN_MARKER);
    expect(content.endsWith("</task-context>")).toBe(true);
    expect(Buffer.byteLength(content, "utf-8")).toBeLessThanOrEqual(700);
  });

  test("a cap smaller than the wrapper emits no malformed payload", async () => {
    const repo = seedTaskRepo("tiny-budget-repo",
      "context_injection:\n  max_total_bytes: 16\n");
    expect(await collectSessionStart(repo, "trellis-implement")).toBe("");
  });

  test("binary and malformed UTF-8 artifacts stay omitted", async () => {
    const repo = seedTaskRepo("binary-repo");
    writeFileSync(join(repo, COMPLEX_TASK, "design.md"), Buffer.from([0xff, 0xfe]));
    writeFileSync(join(repo, COMPLEX_TASK, "implement.md"), Buffer.from("binary\0payload"));
    const content = await collectSessionStart(repo, "trellis-implement");
    expect(content).toContain(PRD_MARKER);
    expect(content).toContain(`${COMPLEX_TASK}/design.md [omitted]`);
    expect(content).toContain(`${COMPLEX_TASK}/implement.md [omitted]`);
    expect(content).toContain("binary or non-UTF-8 artifact");
    expect(content).not.toContain("binary\0payload");
    expect(content).not.toContain("�");
  });

  test("oversized manifests are omitted before parsing their entries", async () => {
    const repo = seedTaskRepo("large-manifest-repo");
    writeUtf8(join(repo, COMPLEX_TASK, "spec.md"), IMPLEMENT_JSONL_MARKER);
    const entry = JSON.stringify({ file: `${COMPLEX_TASK}/spec.md`, reason: "oversized manifest" });
    writeUtf8(join(repo, COMPLEX_TASK, "implement.jsonl"), `${entry}\n${" ".repeat(1024 * 1024)}`);
    const content = await collectSessionStart(repo, "trellis-implement");
    expect(content).toContain(PRD_MARKER);
    expect(content).toContain("manifest exceeds 1048576 byte parse limit");
    expect(content).not.toContain(IMPLEMENT_JSONL_MARKER);
  });
});
