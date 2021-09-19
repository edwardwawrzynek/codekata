import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { GameComponentProps } from './Game';
import './ThreeMensMorris.css';

export default function ThreeMensMorris(props: GameComponentProps) {
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

        // draw horizontal lines
        for(let y = 0; y < 3; y++) {
          for(let i = 0; i < 2; i++) {
            ctx.beginPath();
            ctx.moveTo((i + 0.5) * cellSize + radius, (y + 0.5) * cellSize);
            ctx.lineTo((i + 1.5) * cellSize - radius, (y + 0.5) * cellSize);
            ctx.stroke();
          }
        }
        // draw vertical lines
        for(let x = 0; x < 3; x++) {
          for(let i = 0; i < 2; i++) {
            ctx.beginPath();
            ctx.moveTo((x + 0.5) * cellSize, (i + 0.5) * cellSize + radius);
            ctx.lineTo((x + 0.5) * cellSize, (i + 1.5) * cellSize - radius);
            ctx.stroke();
          }
        }
        // draw diagonal lines
        const diagOff = radius * 0.5 * Math.sqrt(2);
        for(let d = 0; d < 2; d++) {
          for(let i = 0; i < 2; i++) {
            const p0 = (0.5 + i) * cellSize + diagOff;
            const p1 = (1.5 + i) * cellSize - diagOff;
            const p2 = (2.5 - i) * cellSize - diagOff;
            const p3 = (1.5 - i) * cellSize + diagOff;

            ctx.beginPath();
            ctx.moveTo(d === 0 ? p0 : p2, (0.5 + i) * cellSize + diagOff);
            ctx.lineTo(d === 0 ? p1 : p3, (1.5 + i) * cellSize - diagOff);
            ctx.stroke();
          }
        }

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

    const cellPiece = state[0][cellX + cellY * 3];

    // check if placing a piece is expected
    if(remaining[player] > 0) {
      setMoveSrc(null);
      if(cellPiece === '.') {
        onPlay(`${cellX} ${cellY}`);
      }
    } else {
      // moving a piece is expected
      if(moveSrc !== null) {
        // if the already selected cell is clicked, deselect
        if(cellX === moveSrc[0] && cellY === moveSrc[1]) {
          setMoveSrc(null);
        }
        // if there is a piece at destination, select that instead
        else if(cellPiece !== '.') {
          if(cellPiece === player.toString()) {
            setMoveSrc([cellX, cellY]);
          } else {
            setMoveSrc(null);
          }
        } 
        // otherwise, this is a valid move
        else {
          onPlay(`${moveSrc[0]} ${moveSrc[1]} ${cellX} ${cellY}`);
          setMoveSrc(null);
        }
      } 
      
      else {
        if(cellPiece === player.toString()) {
          setMoveSrc([cellX, cellY]);
        } else {
          setMoveSrc(null);
        }
      }
    }
  }, [canvas, size, moveSrc, onPlay, player, remaining, state]);

  return (
    <div className="ThreeMensMorris">
      <canvas ref={canvas} width={400} height={400} tabIndex={1} onClick={onClick}></canvas>
    </div>
  );
}