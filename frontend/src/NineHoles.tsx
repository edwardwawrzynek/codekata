import React, { useEffect, useRef, useState } from 'react';
import Game, { GameComponentProps } from './Game';
import './NineHoles.css';

export default function NineHoles(props: GameComponentProps) {
  const canvas = useRef<HTMLCanvasElement | null>(null);
  const [ctx, setCtx] = useState<CanvasRenderingContext2D | null>(null);

  // get canvas context
  useEffect(() => {
    if(canvas.current !== null) {
      setCtx(canvas.current.getContext("2d"));
    }
  }, [canvas]);

  useEffect(() => {
    if(ctx === null || canvas.current === null) return;
    const [width, height] = [canvas.current.width, canvas.current.height];
    const size = Math.min(width, height);

    ctx.clearRect(0, 0, width, height);
    // draw played pieces
    for(let y = 0; y < 3; y++) {
      for(let x = 0; x < 3; x++) {
        const cell = props.state[0][y * 3 + x];

        ctx.strokeStyle = "black";

        const radius = Math.max(size / 15, 10);

        // draw pieces
        if(cell === '0' || cell === '1') {
          const color = props.gameCfg.playerColors[+cell].bg;
          ctx.fillStyle = color;
          ctx.beginPath();
          ctx.arc((x + 0.5) * (size / 3), (y + 0.5) * (size / 3), radius, 0, 2 * Math.PI);
          ctx.fill();
        }

        // draw empty circle
        ctx.beginPath();
        ctx.arc((x + 0.5) * (size / 3), (y + 0.5) * (size / 3), radius, 0, 2 * Math.PI);
        ctx.stroke();
      }
    }

    // draw remaining pieces
    const remaining = [3, 3];
    for(let y = 0; y < 3; y++) {
      for(let x = 0; x < 3; x++) {
        const cell = props.state[0][y * 3 + x];
        if(cell === '0' || cell === '1') {
          remaining[+cell]--;
        }
      }
    }

    for(let r = 0; r < 2; r++) {
      const radius = size / 64;

      for(let i = 0; i < remaining[r]; i++) {
        ctx.fillStyle = props.gameCfg.playerColors[r].bg;
        ctx.beginPath();
        ctx.arc(r == 0 ? radius * 2 : width - radius * 2, (i + 0.5) * radius * 4, radius, 0, 2 * Math.PI);
        ctx.fill();
      }
    }

  }, [ctx, props.state]);

  return (
    <div className="nineHoles">
      <canvas ref={canvas} width={400} height={400}></canvas>
    </div>
  );
}