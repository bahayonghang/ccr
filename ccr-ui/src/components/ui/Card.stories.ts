import type { Meta, StoryObj } from '@storybook/vue3-vite';
import Card from './Card.vue';

const meta = {
    title: 'UI/Card',
    component: Card,
    tags: ['autodocs'],
    argTypes: {
        variant: {
            control: 'select',
            options: ['base', 'elevated', 'glass', 'outline']
        },
        hover: { control: 'boolean' },
        glow: { control: 'boolean' },
    },
    args: {
        variant: 'elevated',
        hover: true,
        glow: false,
    },
} satisfies Meta<typeof Card>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
    args: {
        default: '<div class="p-6">Content goes here</div>',
    },
};

export const Glass: Story = {
    args: {
        variant: 'glass',
        glow: true,
        default: '<div class="p-6 text-white">Glass Card with Glow</div>',
    },
    parameters: {
        backgrounds: { default: 'dark' },
    },
};

export const Neumorphic: Story = {
    args: {
        variant: 'elevated',
        hover: true,
        default: `
      <div class="p-6 flex flex-col gap-2">
        <h3 class="font-bold text-lg">Neo Card</h3>
        <p class="text-sm opacity-80">Hover to see the elevation lift.</p>
      </div>
    `,
    },
};
