export type LayoutMode = 'minimal' | 'balanced' | 'full';

export interface LayoutSpec {
  startWidth: string;
  recentWidth: string;
  learnWidth: string; // '0%' if hidden
  showLearn: boolean;
}

export function computeLayout(mode: LayoutMode, hasRecents: boolean): LayoutSpec {
  if (!hasRecents) {
    return { startWidth: '33.33%', recentWidth: '33.33%', learnWidth: '33.33%', showLearn: true };
  }
  switch (mode) {
    case 'minimal':
      return { startWidth: '20%', recentWidth: '80%', learnWidth: '0%', showLearn: false };
    case 'full':
      return { startWidth: '20%', recentWidth: '60%', learnWidth: '20%', showLearn: true };
    case 'balanced':
    default:
      return { startWidth: '15%', recentWidth: '70%', learnWidth: '15%', showLearn: true };
  }
}

