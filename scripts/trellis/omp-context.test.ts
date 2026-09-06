import { afterAll, describe, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import registerTrellis from "../../.omp/extensions/trellis/index.ts";

const PRD_MARKER = "OMP_CTX_PRD_MARKER";
const DESIGN_MARKER = "OMP_CTX_DESIGN_MARKER";
const IMPLEMENT_MD_MARKER = "OMP_CTX_IMPLEMENT_MD_MARKER";
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

async function collectSessionStart(cwd: string, role: SessionRole): Promise<string> {
  const previousBlocked = process.env.PI_BLOCKED_AGENT;
  try {
    if (role === "main") {
      delete process.env.PI_BLOCKED_AGENT;
    } else {
      process.env.PI_BLOCKED_AGENT = role;
    }

    const mock = createMockApi();
    registerTrellis(mock.api as Parameters<typeof registerTrellis>[0]);

    const sessionStart = mock.handlers.get("session_start");
    if (!sessionStart) {
      throw new Error("session_start was not registered");
    }

    await sessionStart(
      {} as never,
      {
        cwd,
        sessionManager: {
          getSessionId: () => SESSION_ID,
        },
        ui: {
          notify: () => {},
        },
      } as never,
    );

    return mock.contents.join("\n");
  } finally {
    if (previousBlocked === undefined) {
      delete process.env.PI_BLOCKED_AGENT;
    } else {
      process.env.PI_BLOCKED_AGENT = previousBlocked;
    }
  }
}

function seedComplexRepo(): void {
  writeSession(complexRepo, COMPLEX_TASK);
  writeTaskJson(complexRepo, COMPLEX_TASK, "OMP complex fixture");
  writeUtf8(join(complexRepo, COMPLEX_TASK, "prd.md"), `${PRD_MARKER}\n`);
  writeUtf8(join(complexRepo, COMPLEX_TASK, "design.md"), `${DESIGN_MARKER}\n`);
  writeUtf8(join(complexRepo, COMPLEX_TASK, "implement.md"), `${IMPLEMENT_MD_MARKER}\n`);
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
      expect(content, `${role} missing PRD heading`).toContain("## PRD");
      expect(content, `${role} missing Design heading`).toContain("## Design");
      expect(content, `${role} missing Implement heading`).toContain("## Implement");
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
    expect(content).toContain("## PRD");
    expect(content).not.toContain(DESIGN_MARKER);
    expect(content).not.toContain(IMPLEMENT_MD_MARKER);
    expect(content).not.toContain("## Design");
    expect(content).not.toContain("## Implement");
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
});
