//basic motor control (simplified), specification found at:
//can use negative numbers to move toio backwards
//https://toio.github.io/toio-spec/en/docs/ble_motor#motor-control
void motorBasic(int cubeId, int leftspeed, int rightspeed) {
  int hostId = cubeId/cubesPerHost;
  int actualcubeid = cubeId % cubesPerHost;
  OscMessage msg = new OscMessage("/motorbasic");
  msg.add(actualcubeid);
  if (leftspeed < 0) {
    msg.add(0x01);
  } else {
    msg.add(0x02);
  }
  msg.add(leftspeed);
    if (rightspeed < 0) {
    msg.add(0x01);
  } else {
    msg.add(0x02);
  }
  msg.add(rightspeed);
  oscP5.send(msg, server[hostId]);
}

//basic motor control with duration, specification found at:
//can use negative numbers to move toio backwards
//https://toio.github.io/toio-spec/en/docs/ble_motor#motor-control
void motorDuration(int cubeId, int leftspeed, int rightspeed, int duration) {
  int hostId = cubeId/cubesPerHost;
  int actualcubeid = cubeId % cubesPerHost;
  OscMessage msg = new OscMessage("/motorduration");
  msg.add(actualcubeid);
  if (leftspeed < 0) {
    msg.add(0x01);
  } else {
    msg.add(0x02);
  }
  msg.add(leftspeed);
    if (rightspeed < 0) {
    msg.add(0x01);
  } else {
    msg.add(0x02);
  }
  msg.add(rightspeed);
  msg.add(duration);
  oscP5.send(msg, server[hostId]);
}

//motor control with target specified, specification found at:
//https://toio.github.io/toio-spec/en/docs/ble_motor#motor-control-with-target-specified
void motorTarget(int cubeId, int control, int timeout, int mode, int maxspeed, int speedchange,  int x, int y, int theta){
  int hostId = cubeId/cubesPerHost;
  int actualcubeid = cubeId % cubesPerHost;
  OscMessage msg = new OscMessage("/motortarget");
  msg.add(actualcubeid);
  msg.add(control);
  msg.add(timeout);
  msg.add(mode);
  msg.add(maxspeed);
  msg.add(speedchange);
  msg.add(x);
  msg.add(y);
  msg.add(theta);
  oscP5.send(msg, server[hostId]);
}

void motorTarget(int cubeId, int mode, int x, int y, int theta){
  int hostId = cubeId/cubesPerHost;
  int actualcubeid = cubeId % cubesPerHost;
  OscMessage msg = new OscMessage("/motortarget");
  msg.add(actualcubeid);
  msg.add(0);
  msg.add(0);
  msg.add(mode);
  msg.add(80);
  msg.add(0);
  msg.add(x);
  msg.add(y);
  msg.add(theta);
  oscP5.send(msg, server[hostId]);
}

//motor control with multiple targets specified, specification found at:
//https://toio.github.io/toio-spec/en/docs/ble_motor#motor-control-with-target-specified
//targets should be formatted as {x, y, theta} or {x, y}. Unless specified, theta = 0
void motorMultiTarget(int cubeId, int control, int timeout, int mode, int maxspeed, int speedchange,  int[][] targets){
  int hostId = cubeId/cubesPerHost;
  int actualcubeid = cubeId % cubesPerHost;
  OscMessage msg = new OscMessage("/multitarget");
  msg.add(actualcubeid);
  msg.add(control);
  msg.add(timeout);
  msg.add(mode);
  msg.add(maxspeed);
  msg.add(speedchange);
  
  for (int i = 0; i < targets.length; i++) {
    for (int j = 0; j < targets[i].length; j++) {
      msg.add(targets[i][j]);
    }
    
    if (targets[i].length == 2) {
      msg.add(0);
    }
  }
  oscP5.send(msg, server[hostId]);
}

//motor control with acceleration specified, specification can be found at:
//https://toio.github.io/toio-spec/en/docs/ble_motor#motor-control-with-acceleration-specified
void motorAcceleration(int cubeId, int speed, int a, int rotateVelocity, int rotateDir, int dir, int priority, int duration){
  OscMessage msg = new OscMessage("/motoracceleration");
  int hostId = cubeId/cubesPerHost;
  int actualcubeid = cubeId % cubesPerHost;
  msg.add(actualcubeid);
  msg.add(speed);
  msg.add(a);
  msg.add(rotateVelocity);
  msg.add(rotateDir);
  msg.add(dir);
  msg.add(priority);
  msg.add(duration);
  oscP5.send(msg, server[hostId]);
  oscP5.send(msg, server[hostId]);
}

//activating LED on bottom of toio, specification can be found at:
//https://toio.github.io/toio-spec/en/docs/ble_light
void lightLed(int cubeId, int duration, int red, int green, int blue) {
  int hostId = cubeId/cubesPerHost;
  int actualcubeid = cubeId % cubesPerHost;
  OscMessage msg = new OscMessage("/led");
  msg.add(actualcubeid);
  msg.add(duration);
  msg.add(red);
  msg.add(green);
  msg.add(blue);
  oscP5.send(msg, server[hostId]);
}

//activating LED sequence on bottom of toio, specification can be found at:
//https://toio.github.io/toio-spec/en/docs/ble_light
//lights should be formatted as {duration, red, green, blue}
void lightLed(int cubeId, int repetitions, int[][] lights) {
  int hostId = cubeId/cubesPerHost;
  int actualcubeid = cubeId % cubesPerHost;
  OscMessage msg = new OscMessage("/multiLed");
  
  msg.add(actualcubeid);
  msg.add(repetitions);

  for (int i = 0; i < lights.length; i++) {
    for (int j = 0; j < lights[i].length; j++) {
      msg.add(lights[i][j]);
    }
  } 
  oscP5.send(msg, server[hostId]);
}


//play sound effects, specification can be found at:
//https://toio.github.io/toio-spec/en/docs/ble_sound
void soundEffect(int cubeId, int soundeffect, int volume) {
  int hostId = cubeId/cubesPerHost;
  int actualcubeid = cubeId % cubesPerHost;
  OscMessage msg = new OscMessage("/sound");
  msg.add(actualcubeid);
  msg.add(soundeffect);
  msg.add(volume);
  oscP5.send(msg, server[hostId]);
}

//play Midi Note (single), specification can be found at:
//https://toio.github.io/toio-spec/en/docs/ble_sound/#playing-the-midi-note-numbers
void soundMidi(int cubeId, int duration, int noteID, int volume) {
  int hostId = cubeId/cubesPerHost;
  int actualcubeid = cubeId % cubesPerHost;
  OscMessage msg = new OscMessage("/midi");
  msg.add(actualcubeid);
  msg.add(1);
  msg.add(duration);
  msg.add(noteID);
  msg.add(volume);
  oscP5.send(msg, server[hostId]);
}

//play Midi Notes (advanced), specification can be found at:
//https://toio.github.io/toio-spec/en/docs/ble_sound/#playing-the-midi-note-numbers
//targets should be formatted as {duration, noteID, volume} or {duration, noteID}. Unless specified, volume = 255
void soundMidi(int cubeId, int repetitions, int[][] notes) {
  int hostId = cubeId/cubesPerHost;
  int actualcubeid = cubeId % cubesPerHost;
  OscMessage msg = new OscMessage("/midi");
  
  msg.add(actualcubeid);
  msg.add(repetitions);
  
  for (int i = 0; i < notes.length; i++) {
    for (int j = 0; j < notes[i].length; j++) {
      msg.add(notes[i][j]);
    }
    
    if (notes[i].length == 2) {
      msg.add(255);
    }
  }
  oscP5.send(msg, server[hostId]);
}

//request for motion detection information, specification can be found at:
//https://toio.github.io/toio-spec/en/docs/ble_sensor
void motionRequest(int cubeId) {
  int hostId = cubeId/cubesPerHost;
  int actualcubeid = cubeId % cubesPerHost;
  OscMessage msg = new OscMessage("/motion");
  msg.add(actualcubeid);
  oscP5.send(msg, server[hostId]);
}

void motionRequest(int[] cubeId) {
  for (int i = 0; i < cubeId.length; i++) {
    motionRequest(cubeId[i]);
  }
}

//request for magnetic sensor information, specification can be found at:
//https://toio.github.io/toio-spec/en/docs/ble_magnetic_sensor
void magneticRequest(int cubeId) {
  int hostId = cubeId/cubesPerHost;
  int actualcubeid = cubeId % cubesPerHost;
  OscMessage msg = new OscMessage("/magnetic");
  msg.add(actualcubeid);
  oscP5.send(msg, server[hostId]);
}

void magneticRequest(int[] cubeId) {
  for (int i = 0; i < cubeId.length; i++) {
    magneticRequest(cubeId[i]);
  }
}

//request for cube rotation information (in Eulers), specification can be found at:
//https://toio.github.io/toio-spec/en/docs/ble_high_precision_tilt_sensor
void postureRequest(int cubeId) {
    int hostId = cubeId/cubesPerHost;
    int actualcubeid = cubeId % cubesPerHost;
    OscMessage msg = new OscMessage("/postureeuler");
    msg.add(actualcubeid);
    oscP5.send(msg, server[hostId]);
}

//request for cube rotation information, specification can be found at:
//https://toio.github.io/toio-spec/en/docs/ble_high_precision_tilt_sensor
void postureRequest(int cubeId, boolean euler) {
    int hostId = cubeId/cubesPerHost;
    int actualcubeid = cubeId % cubesPerHost;
    OscMessage msg;
    if (euler) {
      msg = new OscMessage("/postureeuler");
    } else {
      msg = new OscMessage("/posturequaternion");
    }
     msg.add(actualcubeid);
     oscP5.send(msg, server[hostId]);
}

void postureRequest(boolean euler, int[] cubeId) {
  for (int i = 0; i < cubeId.length; i++) {
    postureRequest(cubeId[i], euler);
  }
}

int hostIndex(String host) {
  for (int i = 0; i < hosts.length; i++) {
    if (host.equals(hosts[i])) {
      return i;
    }
  }
  return -1;
}

void oscEvent(OscMessage msg) {
  int hostId = hostIndex(msg.address( ).substring(1));
  int id = (cubesPerHost * hostId) + msg.get(0).intValue();
  
  if (msg.checkAddrPattern("/position")) {
    //this collects position information  
    int posx = msg.get(1).intValue();
    int posy = msg.get(2).intValue();
    int postheta = msg.get(3).intValue();
    
    cubes[id].onPositionUpdate(posx, posy, postheta);
  } 
  
  else if (msg.checkAddrPattern("/battery")) {
    //this collects battery value information
    int battery = msg.get(1).intValue();
    
    cubes[id].onBatteryUpdate(battery);
  }
  
  else if (msg.checkAddrPattern("/motion")) {
    //this collects motion sensor information
    int flatness = msg.get(1).intValue();
    int hit = msg.get(2).intValue();
    int double_tap = msg.get(3).intValue();
    int face_up = msg.get(4).intValue();
    int shake_level = msg.get(5).intValue();
    
    cubes[id].onMotionUpdate(flatness, hit, double_tap, face_up, shake_level);
  } 
  
  else if (msg.checkAddrPattern("/magnetic")) {
    //this collects magnetic sensor information
    int state = msg.get(1).intValue();
    int strength = msg.get(2).intValue();
    int forcex = msg.get(3).intValue();
    int forcey = msg.get(4).intValue();
    int forcez = msg.get(5).intValue();
    
    cubes[id].onMagneticUpdate(state, strength, forcex, forcey, forcez);
  }
  
  else if (msg.checkAddrPattern("/postureeuler")) {
    //this collects posture sensor information (in eulers)
    int roll = msg.get(1).intValue();
    int pitch = msg.get(2).intValue();
    int yaw = msg.get(3).intValue();
    
    cubes[id].onPostureUpdate(roll, pitch, yaw);
  } 
  
  else if (msg.checkAddrPattern("/posturequaternion")) {
    //this collects posture sensor information (in quaternion)
    int w = msg.get(1).intValue();
    int x = msg.get(2).intValue();
    int y = msg.get(3).intValue();
    int z = msg.get(4).intValue();
    
    cubes[id].onPostureUpdate(w, x, y, z);
  } 
  
  else if (msg.checkAddrPattern("/button")) {
    //this collects button information
    int pressValue = msg.get(1).intValue();
    
    if (pressValue == 0) {
      cubes[id].onButtonDown();
    } else {
      cubes[id].onButtonUp();
    } 
  }
  
  else if (msg.checkAddrPattern("/motorresponse")) {
    println("Motor Response Received");
    
    //this collects button information
    int control = msg.get(1).intValue();
    int response = msg.get(2).intValue();
    
    cubes[id].onMotorResponse(control, response);
  }
}
