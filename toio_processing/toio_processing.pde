import oscP5.*;
import netP5.*;


//constants
//The soft limit on how many toios a laptop can handle is in the 10-12 range
//the more toios you connect to, the more difficult it becomes to sustain the connection
int nCubes = 4;
int cubesPerHost = 12;
int maxMotorSpeed = 115;
int xOffset;
int yOffset;

int[] matDimension = {45, 45, 455, 455};
String[] hosts = {"127.0.0.1"};


//for OSC
OscP5 oscP5;
//where to send the commands to
NetAddress[] server;

//we'll keep the cubes here
Cube[] cubes;

void settings() {
  size(1000, 1000);
}


void setup() {
  //create OSC servers
  oscP5 = new OscP5(this, 3333);
  server = new NetAddress[hosts.length];
  for (int i = 0; i < hosts.length; i++) {
    server[i] = new NetAddress(hosts[i], 3334);
  }
  //create cubes
  cubes = new Cube[nCubes];
  for (int i = 0; i< nCubes; ++i) {
    cubes[i] = new Cube(i);
  }

  xOffset = matDimension[0] - 45;
  yOffset = matDimension[1] - 45;

  //do not send TOO MANY PACKETS
  //we'll be updating the cubes every frame, so don't try to go too high
  frameRate(30);

}

void draw() {
  //START TEMPLATE/DEBUG VIEW
  background(255);
  stroke(0);
  long now = System.currentTimeMillis();

  //draw the "mat"
  fill(255);
  rect(matDimension[0] - xOffset, matDimension[1] - yOffset, matDimension[2] - matDimension[0], matDimension[3] - matDimension[1]);

  //draw the cubes
  pushMatrix();
  translate(xOffset, yOffset);
  
  for (int i = 0; i < nCubes; i++) {
    cubes[i].checkActive(now);
    
    if (cubes[i].isActive) {
      pushMatrix();
      translate(cubes[i].x, cubes[i].y);
      fill(0);
      textSize(15);
      text(i, 0, -20);
      noFill();
      rotate(cubes[i].theta * PI/180);
      rect(-10, -10, 20, 20);
      line(0, 0, 20, 0);
      popMatrix();
    }
  }
  popMatrix();
  //END TEMPLATE/DEBUG VIEW
  
  //insert code here
}
