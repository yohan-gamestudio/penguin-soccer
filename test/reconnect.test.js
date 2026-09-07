import {test} from 'node:test';
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {WebSocket} from 'ws';

test('resume preserves player, locked shot, bodies and clocks; expired sessions are rejected', {timeout:45000}, async()=>{
 const server=spawn(process.execPath,['server.js'],{env:{...process.env,PORT:'3110'},stdio:'pipe'});
 const clients=[];
 const sleep=ms=>new Promise(r=>setTimeout(r,ms));
 async function connect(){const ws=new WebSocket('ws://127.0.0.1:3110');ws.messages=[];ws.on('message',raw=>ws.messages.push(JSON.parse(raw)));clients.push(ws);await new Promise(r=>ws.once('open',r));return ws;}
 async function wait(ws,predicate,timeout=4000){const end=Date.now()+timeout;while(Date.now()<end){const i=ws.messages.findIndex(predicate);if(i>=0)return ws.messages.splice(i,1)[0];await sleep(15);}throw Error('Timed out');}
 const send=(ws,m)=>ws.send(JSON.stringify(m));
 try{
  await new Promise((resolve,reject)=>{server.stdout.once('data',resolve);server.once('error',reject);});
  const a=await connect(),b=await connect();send(a,{type:'create',mode:2});const session=await wait(a,m=>m.type==='session');send(b,{type:'join',code:session.code});await wait(a,m=>m.players?.length===2);send(a,{type:'ready'});send(b,{type:'ready'});const before=await wait(a,m=>m.phase==='aim');send(a,{type:'aim',x:.7,y:.2});await wait(b,m=>m.locked?.length===1);a.close();const paused=await wait(b,m=>m.paused);assert.equal(paused.phase,'aim');assert.deepEqual(paused.bodies,before.bodies);assert.ok(paused.players.some(p=>!p.connected));
  const bad=await connect();send(bad,{type:'resume',code:session.code,token:'wrong'});await wait(bad,m=>m.type==='resume_failed');
  await sleep(350);const restored=await connect();send(restored,{type:'resume',code:session.code,token:session.token});const after=await wait(restored,m=>m.type==='state'&&!m.paused);assert.equal(after.you,before.you);assert.deepEqual(after.bodies,before.bodies);assert.deepEqual(after.score,before.score);assert.deepEqual(after.aims[after.you],{x:.7,y:.2});assert.ok(after.deadline>before.deadline+300);
  // A fresh tab connection can replace the old transport without losing its seat.
  const replacement=await connect();send(replacement,{type:'resume',code:session.code,token:session.token});const replaced=await wait(replacement,m=>m.type==='state');assert.equal(replaced.you,before.you);assert.equal(replaced.paused,false);
  b.messages=[];replacement.close();await wait(b,m=>m.paused);await wait(b,m=>m.phase==='lobby'&&!m.paused&&m.players.length===1,33000);
  const late=await connect();send(late,{type:'resume',code:session.code,token:session.token});await wait(late,m=>m.type==='resume_failed');
 }finally{clients.forEach(ws=>ws.terminate());server.kill();}
});
