import type { Meta, StoryObj } from '@storybook/vue3-vite'
import Card from './Card.vue'

const meta = {
  title: 'UI/Card',
  component: Card,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['default', 'base', 'elevated', 'glass', 'outline', 'neko'],
    },
    padding: {
      control: 'select',
      options: ['none', 'sm', 'md', 'lg'],
    },
    hover: { control: 'boolean' },
    interactive: { control: 'boolean' },
    glowEffect: { control: 'boolean' },
    gradientBorder: { control: 'boolean' },
    pattern: { control: 'boolean' },
  },
  args: {
    variant: 'elevated',
    padding: 'md',
    hover: true,
    interactive: true,
    glowEffect: false,
    gradientBorder: false,
    pattern: false,
  },
} satisfies Meta<typeof Card>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: (args) => ({
    components: { Card },
    setup() {
      return { args }
    },
    template: `
      <Card v-bind="args" class="w-72 min-h-44">
        <div class="space-y-3">
          <p class="text-xs uppercase tracking-[0.25em] text-text-muted">Operator Workbench</p>
          <h3 class="text-xl font-bold text-text-primary">Editorial surface</h3>
          <p class="text-sm text-text-secondary">A shared card primitive for dense control panels with restrained, Anthropic-like hierarchy.</p>
        </div>
      </Card>
    `,
  }),
}

export const Glass: Story = {
  args: {
    variant: 'glass',
    glowEffect: true,
    gradientBorder: true,
  },
  render: (args) => ({
    components: { Card },
    setup() {
      return { args }
    },
    template: `
      <div class="p-10 bg-[#15091e]">
        <Card v-bind="args" class="w-72 min-h-44">
          <div class="space-y-2 text-text-primary">
            <p class="text-xs uppercase tracking-[0.25em] text-text-muted">Overlay</p>
            <h3 class="text-xl font-bold">Glass variant</h3>
            <p class="text-sm text-text-secondary">Gradient borders and glow stay readable in the dark theme.</p>
          </div>
        </Card>
      </div>
    `,
  }),
  parameters: {
    backgrounds: { default: 'dark' },
  },
}

export const LegacyDecorative: Story = {
  args: {
    variant: 'neko',
    pattern: true,
    glowEffect: true,
  },
  render: (args) => ({
    components: { Card },
    setup() {
      return { args }
    },
    template: `
      <div class="pt-4">
        <Card v-bind="args" class="w-72 min-h-44">
          <div class="space-y-3">
            <p class="text-xs uppercase tracking-[0.25em] text-text-muted">Legacy Accent</p>
            <h3 class="text-xl font-bold text-text-primary">Deprecated shell chrome</h3>
            <p class="text-sm text-text-secondary">This variant is legacy-only and should not define new Anthropic-like surfaces.</p>
          </div>
        </Card>
      </div>
    `,
  }),
}
