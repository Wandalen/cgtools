// Pong-style scene script: loops + branches + F32x2 vector arithmetic
// simulate ball/paddle motion; emit_frame() is a host callback recording
// one frame per tick.
//
// Simplification: paddles always intercept the ball at the court edges
// (no miss/score logic) — this is an animation-glue demo, not a full game.
//
// Top-level-bindings convention: only config bindings live here; all
// imperative code (the loop, branches, mutation) lives inside main(). Rhai
// functions can't see outer-scope let/const at all, so the mutable
// simulation state (ball_pos, ball_vel, paddle_*_y) is declared fresh
// inside main(), seeded from these top-level start values.

let court = f32x2( 200.0, 100.0 );
let paddle_speed = 2.0;

let ball_start_pos = f32x2( 100.0, 50.0 );
let ball_start_vel = f32x2( 2.5, 1.7 );

let paddle_start_y = 50.0;

let ticks = 40;

fn main( court, paddle_speed, ball_start_pos, ball_start_vel, paddle_start_y, ticks )
{
  let ball_pos = ball_start_pos;
  let ball_vel = ball_start_vel;

  let paddle_left_y = paddle_start_y;
  let paddle_right_y = paddle_start_y;

  for i in 0..ticks
  {
    ball_pos = ball_pos + ball_vel;

    // bounce off top / bottom walls
    if ball_pos.y <= 0.0 || ball_pos.y >= court.y
    {
      ball_vel = f32x2( ball_vel.x, -ball_vel.y );
    }

    // bounce off left / right paddles
    if ball_pos.x <= 0.0 || ball_pos.x >= court.x
    {
      ball_vel = f32x2( -ball_vel.x, ball_vel.y );
    }

    // paddles track the ball's y with a clamped step
    if paddle_left_y < ball_pos.y
    {
      paddle_left_y += paddle_speed;
    }
    else if paddle_left_y > ball_pos.y
    {
      paddle_left_y -= paddle_speed;
    }

    if paddle_right_y < ball_pos.y
    {
      paddle_right_y += paddle_speed;
    }
    else if paddle_right_y > ball_pos.y
    {
      paddle_right_y -= paddle_speed;
    }

    emit_frame( i, ball_pos, paddle_left_y, paddle_right_y );
  }

  "done"
}

main( court, paddle_speed, ball_start_pos, ball_start_vel, paddle_start_y, ticks )
