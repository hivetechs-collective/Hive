import React, { useState, useRef, useEffect, ReactNode } from 'react';

interface Panel {
  id: string;
  defaultSize: number | 'flex';
  minSize?: number;
  maxSize?: number;
  content: ReactNode;
}

interface ResizablePanelsProps {
  panels: Panel[];
  direction?: 'horizontal' | 'vertical';
}

export const ResizablePanels: React.FC<ResizablePanelsProps> = ({ 
  panels, 
  direction = 'horizontal' 
}) => {
  const [sizes, setSizes] = useState<number[]>(() => {
    const flexCount = panels.filter(p => p.defaultSize === 'flex').length;
    const totalFixed = panels.reduce((sum, p) => 
      typeof p.defaultSize === 'number' ? sum + p.defaultSize : sum, 0
    );
    
    return panels.map(p => {
      if (typeof p.defaultSize === 'number') return p.defaultSize;
      // Distribute remaining space among flex panels
      return flexCount > 0 ? (100 - totalFixed) / flexCount : 100;
    });
  });

  const containerRef = useRef<HTMLDivElement>(null);
  const isDragging = useRef(false);
  const dragIndex = useRef(-1);

  const handleMouseDown = (index: number) => (e: React.MouseEvent) => {
    e.preventDefault();
    isDragging.current = true;
    dragIndex.current = index;
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = direction === 'horizontal' ? 'col-resize' : 'row-resize';
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDragging.current || !containerRef.current) return;

    const container = containerRef.current;
    const rect = container.getBoundingClientRect();
    const pos = direction === 'horizontal' 
      ? ((e.clientX - rect.left) / rect.width) * 100
      : ((e.clientY - rect.top) / rect.height) * 100;

    const index = dragIndex.current;
    const newSizes = [...sizes];
    
    // Calculate the change in size
    const totalBefore = newSizes.slice(0, index + 1).reduce((a, b) => a + b, 0);
    const delta = pos - totalBefore;
    
    // Apply constraints
    const panel1 = panels[index];
    const panel2 = panels[index + 1];
    
    const newSize1 = newSizes[index] + delta;
    const newSize2 = newSizes[index + 1] - delta;
    
    // Check min/max constraints
    const min1 = panel1.minSize || 50;
    const max1 = panel1.maxSize || Infinity;
    const min2 = panel2.minSize || 50;
    const max2 = panel2.maxSize || Infinity;
    
    if (newSize1 >= min1 && newSize1 <= max1 && newSize2 >= min2 && newSize2 <= max2) {
      newSizes[index] = newSize1;
      newSizes[index + 1] = newSize2;
      setSizes(newSizes);
    }
  };

  const handleMouseUp = () => {
    isDragging.current = false;
    dragIndex.current = -1;
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = '';
  };

  useEffect(() => {
    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, []);

  const isHorizontal = direction === 'horizontal';

  return (
    <div 
      ref={containerRef}
      className="resizable-panels" 
      style={{ 
        flexDirection: isHorizontal ? 'row' : 'column',
        height: '100%',
        width: '100%',
      }}
    >
      {panels.map((panel, index) => (
        <React.Fragment key={panel.id}>
          <div
            className="resizable-panel"
            style={{
              [isHorizontal ? 'width' : 'height']: 
                panel.defaultSize === 'flex' ? `${sizes[index]}%` : `${sizes[index]}px`,
              flex: panel.defaultSize === 'flex' ? 1 : undefined,
              display: 'flex',
              flexDirection: 'column',
              overflow: 'hidden',
            }}
          >
            {panel.content}
          </div>
          {index < panels.length - 1 && (
            <div
              className={`resize-handle resize-handle-${isHorizontal ? 'vertical' : 'horizontal'}`}
              onMouseDown={handleMouseDown(index)}
              style={{
                [isHorizontal ? 'width' : 'height']: '1px',
                background: 'var(--border-color)',
                cursor: isHorizontal ? 'col-resize' : 'row-resize',
                flexShrink: 0,
              }}
            />
          )}
        </React.Fragment>
      ))}
    </div>
  );
};