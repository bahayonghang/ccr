export type BindingStep = { command: string; args: string[]; exports?: boolean }
export type BindingRunner = (step: BindingStep) => number | Promise<number>
export const generatedRoot: string
export const generationSteps: BindingStep[]
export const normalizationStep: BindingStep
export function runBindingStep(step: BindingStep): number
export function generateContents(directory: string, runStep?: BindingRunner): Promise<number>
export function generateBindings(options?: { directory?: string; runStep?: BindingRunner }): Promise<number>
