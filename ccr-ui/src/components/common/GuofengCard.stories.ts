import type { Meta, StoryObj } from '@storybook/vue3-vite'
import GuofengCard from './GuofengCard.vue'
import { tokens } from '../../styles/tokens'

const meta: Meta<typeof GuofengCard> = {
  title: 'Common/GuofengCard',
  component: GuofengCard,
  tags: ['autodocs'],
  argTypes: {
    variant: { control: 'select', options: ['default', 'glass'] },
    padding: { control: 'select', options: ['none', 'sm', 'md', 'lg'] },
    glowColor: { control: 'select', options: Object.keys(tokens.colors.accent) },
    interactive: { control: 'boolean' },
    disabled: { control: 'boolean' },
  },
  args: {
    variant: 'default',
    padding: 'md',
    interactive: true,
    glowEffect: true,
  },
}

export default meta
type Story = StoryObj<typeof GuofengCard>

export const Default: Story = {
  render: (args) => ({
    components: { GuofengCard },
    setup() {
      return { args }
    },
    template: `
      <GuofengCard v-bind="args" class="w-64 h-40 flex items-center justify-center">
        <span class="text-lg font-bold">Default Card</span>
      </GuofengCard>
    `,
  }),
}

export const GlassVariant: Story = {
  args: {
    variant: 'glass',
    gradientBorder: true,
  },
  render: (args) => ({
    components: { GuofengCard },
    setup() {
      return { args }
    },
    template: `
      <div class="p-10 bg-gray-900">
        <GuofengCard v-bind="args" class="w-64 h-40 flex items-center justify-center text-white">
          <span class="text-lg font-bold">Glass Card</span>
        </GuofengCard>
      </div>
    `,
  }),
}

export const InteractiveHover: Story = {
  args: {
    interactive: true,
    glowEffect: true,
    glowColor: 'primary',
  },
  render: (args) => ({
    components: { GuofengCard },
    setup() {
      return { args }
    },
    template: `
      <GuofengCard v-bind="args" class="w-64 h-40 flex items-center justify-center">
        <div class="text-center">
          <p class="font-bold mb-2">Hover Me</p>
          <p class="text-sm text-gray-500">See breathing effect</p>
        </div>
      </GuofengCard>
    `,
  }),
}

export const Disabled: Story = {
  args: {
    disabled: true,
    interactive: true,
  },
  render: (args) => ({
    components: { GuofengCard },
    setup() {
      return { args }
    },
    template: `
      <GuofengCard v-bind="args" class="w-64 h-40 flex items-center justify-center">
        <span class="text-lg font-bold text-gray-400">Disabled</span>
      </GuofengCard>
    `,
  }),
}
