import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { GameComponentProps } from './Game';
import './NineHoles.css';

export default function NineHoles(props: GameComponentProps) {
  const canvas = useRef<HTMLCanvasElement | null>(null);
  const [ctx, setCtx] = useState<CanvasRenderingContext2D | null>(null);
  const [moveSrc, setMoveSrc] = useState<[number, number] | null>(null);

  const {onPlay, player, gameCfg, state} = props;

  // get canvas context
  useEffect(() => {
    if(canvas.current !== null) {
      setCtx(canvas.current.getContext("2d"));
    }
  }, [canvas]);

  // canvas size
  const [width, height] = [canvas.current?.width ?? 0, canvas.current?.height ?? 0];
  const size = Math.min(width ?? 0, height ?? 0);

  // remaining pieces for each player
  const remaining = useMemo(() => {
    let remaining = [3, 3];
    for(let y = 0; y < 3; y++) {
      for(let x = 0; x < 3; x++) {
        const cell = state[0][y * 3 + x];
        if(cell === '0' || cell === '1') {
          remaining[+cell]--;
        }
      }
    }
    return remaining;
  }, [state]);

  // draw game
  useEffect(() => {
    if(ctx === null || canvas.current === null) return;

    ctx.clearRect(0, 0, width, height);
    // draw played pieces
    for(let y = 0; y < 3; y++) {
      for(let x = 0; x < 3; x++) {
        const cell = state[0][y * 3 + x];
        const cellSize = size / 3;
        const radius = Math.max(size / 15, 10);

        // if current selected cell, color background
        if(moveSrc !== null && x === moveSrc[0] && y === moveSrc[1]) {
          ctx.fillStyle = "yellow";
          ctx.fillRect((x + 0.5) * cellSize - radius * 2, (y + 0.5) * cellSize - radius * 2, radius * 4, radius * 4);
        }

        ctx.strokeStyle = "black";

        // draw pieces
        if(cell === '0' || cell === '1') {
          const color = gameCfg.playerColors[+cell].bg;
          ctx.fillStyle = color;
          ctx.beginPath();
          ctx.arc((x + 0.5) * cellSize, (y + 0.5) * cellSize, radius, 0, 2 * Math.PI);
          ctx.fill();
        }

        // draw empty circle
        ctx.beginPath();
        ctx.arc((x + 0.5) * cellSize, (y + 0.5) * cellSize, radius, 0, 2 * Math.PI);
        ctx.stroke();
      }
    }

    // draw remaining pieces
    for(let r = 0; r < 2; r++) {
      const radius = size / 64;

      for(let i = 0; i < remaining[r]; i++) {
        ctx.fillStyle = gameCfg.playerColors[r].bg;
        ctx.beginPath();
        ctx.arc(r === 0 ? radius * 2 : width - radius * 2, (i + 0.5) * radius * 4, radius, 0, 2 * Math.PI);
        ctx.fill();
      }
    }

  }, [ctx, state, height, width, size, gameCfg.playerColors, remaining, moveSrc]);

  // handle click
  const onClick = useCallback((e: React.MouseEvent) => {
    if(player === null) return;

    // get clicked cell
    if(canvas.current === null) return;
    const canvasLoc = canvas.current.getBoundingClientRect();
    const [x, y] = [(e.clientX - canvasLoc.left) / size, (e.clientY - canvasLoc.top) / size];
    const [cellX, cellY] = [Math.floor(x * 3), Math.floor(y * 3)];

    // check if placing a piece is expected
    if(remaining[player] > 0) {
      setMoveSrc(null);
      onPlay(`${cellX} ${cellY}`);
    } else {
      // moving a piece is expected
      if(moveSrc !== null) {
        if(cellX === moveSrc[0] && cellY === moveSrc[1]) {
          setMoveSrc(null);
        } else {
          onPlay(`${moveSrc[0]} ${moveSrc[1]} ${cellX} ${cellY}`);
          setMoveSrc(null);
        }
      } else {
        setMoveSrc([cellX, cellY]);
      }
    }
  }, [canvas, size, moveSrc, onPlay, player, remaining]);

  return (
    <div className="nineHoles">
      <canvas ref={canvas} width={400} height={400} tabIndex={1} onClick={onClick}></canvas>
    </div>
  );
}