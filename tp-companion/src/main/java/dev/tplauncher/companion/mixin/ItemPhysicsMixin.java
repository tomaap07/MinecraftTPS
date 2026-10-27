package dev.tplauncher.companion.mixin;
import dev.tplauncher.companion.Profile;
import net.minecraft.client.render.entity.ItemEntityRenderer;
import net.minecraft.client.render.item.ItemRenderer;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.client.render.VertexConsumerProvider;
import net.minecraft.entity.ItemEntity;
import net.minecraft.util.math.RotationAxis;
import net.minecraft.util.math.random.Random;
import org.spongepowered.asm.mixin.*;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(ItemEntityRenderer.class)
public abstract class ItemPhysicsMixin{
 @Shadow @Final private ItemRenderer itemRenderer;
 @Inject(method="render(Lnet/minecraft/entity/ItemEntity;FFLnet/minecraft/client/util/math/MatrixStack;Lnet/minecraft/client/render/VertexConsumerProvider;I)V",at=@At("HEAD"),cancellable=true)
 private void tp$items(ItemEntity item,float yaw,float delta,MatrixStack matrices,VertexConsumerProvider vertices,int light,CallbackInfo ci){
  if(!Profile.get().enabled("item_physics"))return;
  matrices.push();try{
   matrices.translate(0,item.isOnGround()?.07:.18,0);
   matrices.multiply(RotationAxis.POSITIVE_Y.rotationDegrees((item.getId()*137)%360));
   matrices.multiply(RotationAxis.POSITIVE_X.rotationDegrees(item.isOnGround()?90:(item.age+delta)*14));
   ItemEntityRenderer.renderStack(itemRenderer,matrices,vertices,light,item.getStack(),Random.create(item.getId()),item.getWorld());
  }finally{matrices.pop();}ci.cancel();
 }
}
