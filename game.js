export const W=1000,H=600;
export function setup(players){return [...players.map((p,i)=>({id:p.id,team:p.team,x:p.team===0?230:770,y:players.length===2?300:(players.filter(q=>q.team===p.team).indexOf(p)===0?220:380),vx:0,vy:0,r:25,m:2})),{id:'ball',x:500,y:300,vx:0,vy:0,r:15,m:0.7}];}
export function step(bodies,dt){
 for(const b of bodies){b.x+=b.vx*dt;b.y+=b.vy*dt;const drag=Math.exp(-1.25*dt);b.vx*=drag;b.vy*=drag;
 if(b.y<b.r){b.y=b.r;b.vy=Math.abs(b.vy)*.8;}if(b.y>H-b.r){b.y=H-b.r;b.vy=-Math.abs(b.vy)*.8;}
 if(b.id==='ball'&&b.y>220&&b.y<380){if(b.x<0)return 1;if(b.x>W)return 0;}else{if(b.x<b.r){b.x=b.r;b.vx=Math.abs(b.vx)*.8;}if(b.x>W-b.r){b.x=W-b.r;b.vx=-Math.abs(b.vx)*.8;}}
 }
 for(let i=0;i<bodies.length;i++)for(let j=i+1;j<bodies.length;j++){const a=bodies[i],b=bodies[j],dx=b.x-a.x,dy=b.y-a.y,d=Math.hypot(dx,dy),r=a.r+b.r;if(d>=r)continue;const nx=d?dx/d:1,ny=d?dy/d:0,ia=1/a.m,ib=1/b.m,over=(r-d)/(ia+ib);a.x-=nx*over*ia;a.y-=ny*over*ia;b.x+=nx*over*ib;b.y+=ny*over*ib;const v=(b.vx-a.vx)*nx+(b.vy-a.vy)*ny;if(v<0){const impulse=-(1+.85)*v/(ia+ib);a.vx-=impulse*nx*ia;a.vy-=impulse*ny*ia;b.vx+=impulse*nx*ib;b.vy+=impulse*ny*ib;}}
 return null;
}
