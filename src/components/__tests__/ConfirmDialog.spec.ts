import { describe, expect, it } from 'vitest';
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';
import ConfirmDialog from '@/components/ConfirmDialog.vue';
import zhCN from '@/i18n/zh-CN.json';

interface DetailItem {
  path: string;
  size: number;
}

interface DialogProps {
  visible: boolean;
  title: string;
  message: string;
  detailItems?: DetailItem[];
  confirmText?: string;
  cancelText?: string;
}

// 使用固定 zh-CN locale 的独立 i18n 实例,避免依赖 jsdom 的 navigator.language。
function makeI18n() {
  return createI18n({
    legacy: false,
    locale: 'zh-CN',
    fallbackLocale: 'zh-CN',
    messages: { 'zh-CN': zhCN },
  });
}

function mountDialog(props: DialogProps) {
  return mount(ConfirmDialog, {
    props,
    global: { plugins: [makeI18n()] },
  });
}

describe('ConfirmDialog', () => {
  it('does not render the card when visible is false', () => {
    // 根节点使用 v-if="visible",visible=false 时不渲染卡片。
    const wrapper = mountDialog({ visible: false, title: 'T', message: 'M' });
    expect(wrapper.find('h3').exists()).toBe(false);
  });

  it('renders title and message when visible is true', () => {
    const wrapper = mountDialog({ visible: true, title: '删除确认', message: '将删除 3 项' });
    expect(wrapper.find('h3').text()).toBe('删除确认');
    expect(wrapper.find('p').text()).toBe('将删除 3 项');
  });

  it('emits "confirm" when the confirm button is clicked', async () => {
    const wrapper = mountDialog({
      visible: true,
      title: 'T',
      message: 'M',
      confirmText: '确认',
      cancelText: '取消',
    });
    const buttons = wrapper.findAll('button');
    expect(buttons).toHaveLength(2);
    // 模板中取消按钮在前、确认按钮在后,故 buttons[1] 为确认按钮。
    await buttons[1].trigger('click');
    expect(wrapper.emitted('confirm')).toHaveLength(1);
  });

  it('emits "cancel" when the cancel button is clicked', async () => {
    const wrapper = mountDialog({
      visible: true,
      title: 'T',
      message: 'M',
      confirmText: '确认',
      cancelText: '取消',
    });
    const buttons = wrapper.findAll('button');
    expect(buttons).toHaveLength(2);
    // buttons[0] 为取消按钮。
    await buttons[0].trigger('click');
    expect(wrapper.emitted('cancel')).toHaveLength(1);
  });

  it('renders at most 10 detail items and an overflow summary', () => {
    const items: DetailItem[] = [...Array(12).keys()].map((i) => ({
      path: `/item/${i}`,
      size: 1024 * (i + 1),
    }));
    const wrapper = mountDialog({
      visible: true,
      title: 'T',
      message: 'M',
      detailItems: items,
    });
    // 最多展示 10 条,其余汇总为 "...等 N 项"。
    expect(wrapper.findAll('li')).toHaveLength(10);
    expect(wrapper.text()).toContain('/item/0');
    // 12 - 10 = 2 -> "...等 2 项"
    expect(wrapper.text()).toContain('等 2 项');
  });
});
